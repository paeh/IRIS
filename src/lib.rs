mod tipc_include;
mod iris_error_wrapper;

use tipc_include::*;
use std::os::raw::{c_int, c_void, c_uint};
use iris_error_wrapper::IrisErrorWrapper;
use std::result::{Result};
use std::vec::Vec;
use std::fmt;

pub type IrisResult<T> = Result<T, IrisErrorWrapper>;

pub const TOPO_SERVICE: u32 = TIPC_TOP_SRV;

pub enum SockType
{
    Stream = 1,
    Datagram,
    Raw,
    Rdm,
    Sequential,
}

pub enum ScopeType
{
    Cluster = 2,
    Node,
}

pub struct IrisAddressType
{
    pub addr_type: u32,
    pub instance: u32,
    pub node: u32,
}

impl IrisAddressType
{
    pub fn new() -> Self
    {
        IrisAddressType{addr_type: 0, instance: 0, node: 0}
    }
}

impl fmt::Display for IrisAddressType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "({}, {}, {})", self.addr_type, self.instance, self.node)
    }
}

pub enum SockOption
{
    Standard,
    NonBlocking
}

pub struct Subscription
{
    pub service: u32,
    pub instance: u32,
    pub timeout: i32
}

pub struct Connection
{
    socket: c_int,
    own_node: c_uint,
}

impl Connection
{
    pub fn new(sock_type: SockType) -> Connection
    {
        let sock = unsafe { tipc_socket(sock_type as i32) };
        let own_node = unsafe { tipc_own_node() };
        Connection{socket: sock, own_node: own_node}
    }

    pub fn get_socket(&self) -> i32
    {
        self.socket
    }

    fn get_node_for_scope(&self, scope: ScopeType) -> u32
    {
        match scope
        {
            ScopeType::Cluster => 0,
            ScopeType::Node => self.own_node,
        }
    }

    pub fn bind(&self, service_type: u32, lower: u32, upper: u32, scope: ScopeType) -> IrisResult<()>
    {
        let node = self.get_node_for_scope(scope);

        let res = unsafe { tipc_bind(self.socket, service_type, lower, upper, node) };

        match res
        {
            0 => Ok(()),
            -1 => Err(IrisErrorWrapper::new_with_code("Bind failed due to scope error!", -1)),
            _ => Err(IrisErrorWrapper::new("Bind failed!")),
        }
    }

    pub fn recv_from(&self, sock: &mut IrisAddressType, member: &mut IrisAddressType) -> IrisResult<Vec<u8>>
    {
        let buf: [u8; TIPC_MAX_USER_MSG_SIZE as usize] = [0; TIPC_MAX_USER_MSG_SIZE as usize];
        let mut sock_id = tipc_addr { type_ : 0, instance: 0, node: 0 };
        let mut client_id = tipc_addr { type_ : 0, instance: 0, node: 0 };
        let mut err: i32 = 0;

        let res = unsafe { tipc_recvfrom(self.socket,
                                         buf.as_ptr() as *mut c_void,
                                         TIPC_MAX_USER_MSG_SIZE as u64,
                                         &mut sock_id,
                                         &mut client_id,
                                         &mut err) };
        match res
        {
            val if val >= 0 => {
                // store socket information
                sock.addr_type = sock_id.type_;
                sock.instance = sock_id.instance;
                sock.node = sock_id.node;

                //store member id
                member.addr_type = client_id.type_;
                member.instance = client_id.instance;
                member.node = client_id.node;

                Ok(buf[0..res as usize].to_vec())
            },
            _ => Err(IrisErrorWrapper::new("No data was received!")),
        }
    }

    pub fn recv(&self) -> IrisResult<Vec<u8>>
    {
        let buf: [u8; TIPC_MAX_USER_MSG_SIZE as usize] = [0; TIPC_MAX_USER_MSG_SIZE as usize];
        let res = unsafe { tipc_recv(self.socket,
                                     buf.as_ptr() as *mut c_void,
                                     TIPC_MAX_USER_MSG_SIZE as u64,
                                     true) };
        match res
        {
            val if val > 0 => Ok(buf[0..res as usize].to_vec()),
            0 => Err(IrisErrorWrapper::new("No data was received! Could be a event!")),
            _ => Err(IrisErrorWrapper::new("Error during recv!")),
        }
    }

    pub fn connect(&self, service_type: u32, instance: u32, scope: ScopeType) -> IrisResult<()>
    {
        let node = self.get_node_for_scope(scope);
        let addr = tipc_addr { type_: service_type, instance: instance, node: node };

        let res = unsafe { tipc_connect(self.socket, &addr) };

        match res
        {
            0 => Ok(()),
            _ => Err(IrisErrorWrapper::new("Connect failed!")),
        }
    }

    fn get_sock_type(sock_type: u32) -> SockType
    {
        match sock_type
        {
            1 => SockType::Stream,
            2 => SockType::Datagram,
            3 => SockType::Raw,
            4 => SockType::Rdm,
            5 => SockType::Sequential,
            _ => SockType::Rdm,
        }
    }

    pub fn accept(&self) -> IrisResult<IrisAddressType>
    {
        let mut addr = tipc_addr { type_: 0, instance: 0, node: 0 };
        let res = unsafe { tipc_accept(self.socket, &mut addr) };

        match res
        {
            0 => Ok(IrisAddressType{ addr_type: addr.type_, instance: addr.instance, node: addr.node }),
            _ => Err(IrisErrorWrapper::new("Accept failed!")),
        }
    }

    pub fn listen(&self) -> IrisResult<()>
    {
        let res = unsafe{ tipc_listen(self.socket, 0) };

        match res{
            0 => Ok(()),
            _ => Err(IrisErrorWrapper::new("Listen failed!")),
        }
    }

    pub fn send(&self, buf: &Vec<u8>) -> IrisResult<i32>
    {
        let res = unsafe { tipc_send(self.socket, buf.as_ptr() as *const c_void, buf.len() as u64) };

        match res
        {
            _ if buf.len() == (res as usize) => Ok(res),
            _ if buf.len() > (res as usize) => Err(IrisErrorWrapper::new(format!("Data only send partially: {}", res).as_str())),
            _ => Err(IrisErrorWrapper::new("Send failed!")),
        }
    }

    pub fn sendto(&self, buf: &Vec<u8>, addr: &IrisAddressType) -> IrisResult<i32>
    {
        let dest = Connection::make_tipc_addr(addr);
        let res = unsafe { tipc_sendto(self.socket, buf.as_ptr() as *const c_void, buf.len() as u64, &dest) };

        match res
        {
            _ if buf.len() == (res as usize) => Ok(res),
            _ if buf.len() > (res as usize) => Err(IrisErrorWrapper::new(format!("Data only send partially: {}", res).as_str())),
            _ => Err(IrisErrorWrapper::new("Sendto failed!")),
        }
    }

    pub fn multicast(&self, buf: &Vec<u8>, addr: &IrisAddressType) -> IrisResult<i32>
    {
        let dest = Connection::make_tipc_addr(addr);
        let res = unsafe { tipc_mcast(self.socket, buf.as_ptr() as *const c_void, buf.len() as u64, &dest) };

        match res
        {
            val if buf.len() == (val as usize) => Ok(val),
            val if buf.len() > (val as usize) => Err(IrisErrorWrapper::new(format!("Data only send partially: {}", val).as_str())),
            _ => Err(IrisErrorWrapper::new("Multicast failed!")),
        }
    }

    pub fn subscribe(&self, subscription: &Subscription) -> IrisResult<()>
    {
        let res = unsafe { tipc_srv_subscr(self.socket,
                                           subscription.service,
                                           subscription.instance,
                                           subscription.instance,
                                           true,
                                           subscription.timeout) };
        
        match res
        {
            0 => Ok(()),
            _ => Err(IrisErrorWrapper::new("Subscription failed!")),
        }
    }

    pub fn wait_for_service(srv: &Subscription) -> IrisResult<bool>
    {
        let sub = tipc_addr{ type_: srv.service,
                             instance: srv.instance,
                             node: 0 };
        let res = unsafe { tipc_srv_wait(&sub, srv.timeout) };

        match res
        {
            true => Ok(res),
            false => Err(IrisErrorWrapper::new("Service not available in time")),
        }
    }

    pub fn make_non_blocking(&self) -> IrisResult<()>
    {
        let res = unsafe { tipc_sock_non_block(self.socket)};
        
        match res
        {
            val if val == self.socket => Ok(()),
            _ => Err(IrisErrorWrapper::new("Making sockiet non-blocking failed!")),
        }
    }

    fn make_tipc_addr(addr: &IrisAddressType) -> tipc_addr
    {
        tipc_addr{type_: addr.addr_type, instance: addr.instance, node: addr.node}
    }

    pub fn join(&self, addr: &IrisAddressType, events: bool, loopback: bool) -> IrisResult<()>
    {
        let mut group = Connection::make_tipc_addr(addr);
        let res = unsafe{tipc_join(self.socket, &mut group, events, loopback)};

        match res
        {
            0 => Ok(()),
            _ => Err(IrisErrorWrapper::new("Group join failed!")),
        }
    }

    pub fn leave(&self) -> IrisResult<()>
    {
        let res = unsafe{tipc_leave(self.socket)};

        match res
        {
            0 => Ok(()),
            _ => Err(IrisErrorWrapper::new("Group leave failed!")),
        }
    }
}
