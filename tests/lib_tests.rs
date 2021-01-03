use iris::*;
use std::thread;
use std::time::Duration;

const SERVICE_TYPE : u32 = 12345;
const SERVICE_LOWER : u32 = 100;
const SERVICE_UPPER : u32 = 200;
const SERVICE_INSTANCE : u32 = 115;
const GROUP_ID : u32 = 12121;
const GROUP_INSTANCE : u32 = 100;
const SERVER_SCOPE : ScopeType = ScopeType::Cluster;

#[test]
fn test_rdm_socket_creation()
{
    let conn = Connection::new(SockType::Rdm);
    assert_eq!(conn.get_socket() != 0, true);
}

#[test]
fn test_bind_cluster()
{
    let conn = Connection::new(SockType::Rdm);
    assert_eq!(conn.get_socket() != 0, true);

    match conn.bind(SERVICE_TYPE, SERVICE_LOWER+101, SERVICE_UPPER+101, SERVER_SCOPE)
    {
        Err(e) => assert!(false, format!("Test failed: {}", e)),
        Ok(_) => {},
    }
}

#[test]
fn test_connect_to_topo()
{
    let conn = Connection::new(SockType::Sequential);
    assert_eq!(conn.get_socket() != 0, true);

    match conn.connect(TOPO_SERVICE, TOPO_SERVICE, ScopeType::Cluster)
    {
        Err(e) => assert!(false, format!("Test failed: {}", e)),
        Ok(_) => {},
    }

    let sub = Subscription{service: SERVICE_TYPE, instance: SERVICE_INSTANCE, timeout: 30};
    match conn.subscribe(&sub)
    {
        Err(e) => assert!(false, format!("Test failed: {}", e)),
        Ok(_) => {},
    }
}

#[test]
fn test_wait_for_service_and_hello_world()
{
    thread::spawn(||{
        /* Server part */
        thread::sleep(Duration::from_millis(2000));
        let s_conn = Connection::new(SockType::Rdm);
        assert!(s_conn.get_socket() != 0);

        match s_conn.bind(SERVICE_TYPE, SERVICE_LOWER, SERVICE_UPPER, ScopeType::Cluster)
        {
            Err(e) => assert!(false, format!("Test failed: {}", e)),
            Ok(_) => {},
        }

        let mut sender = IrisAddressType::new();
        let mut member = IrisAddressType::new();
        
        match s_conn.recv_from(&mut sender, &mut member)
        {
            Ok(r) => {
                assert!(r[0] == 'a' as u8);
                assert!(r[1] == 's' as u8);
                assert!(r[2] == 'd' as u8);
                assert!(r[3] == 'f' as u8);
            },
            Err(e) => assert!(false, "Test failed: {}", e)
        };

        let resp = vec!('j' as u8, 'k' as u8);
        
        match s_conn.sendto(&resp, &sender,)
        {
            Ok(_) => {},
            Err(e) => assert!(false, "Test failed: {}", e),
        }
    });

    let sub = Subscription{service: SERVICE_TYPE,
                           instance: SERVICE_INSTANCE,
                           timeout: 10000};

    match Connection::wait_for_service(&sub)
    {
        Ok(r) => assert!(r),
        Err(e) => assert!(false, "Test failed: {}", e),
    }

    let data = vec!('a' as u8, 's' as u8, 'd' as u8, 'f' as u8);
    let dest = IrisAddressType{addr_type: SERVICE_TYPE, instance: SERVICE_INSTANCE, node: 0};
    let conn = Connection::new(SockType::Sequential);

    match conn.sendto(&data, &dest)
    {
        Ok(r) => assert!(r == 4),
        Err(e) => assert!(false, "Test failed: {}", e),
    }

    match conn.recv()
    {
        Ok(r) => {
            assert!(r[0] == 'j' as u8);
            assert!(r[1] == 'k' as u8);
        },
        Err(_) => {
            match conn.recv()
            {
                Ok(r) => {
                    assert!(r[0] == 'j' as u8);
                    assert!(r[1] == 'k' as u8);
                },
                Err(e) => assert!(false, "Test failed: {}", e),
            };
        },
    };
}

#[test]
fn test_join_and_group_com()
{
    let conn = Connection::new(SockType::Rdm);
    assert_eq!(conn.get_socket() != 0, true);

    let group = IrisAddressType{addr_type: GROUP_ID, instance: GROUP_INSTANCE, node: 0};

    match conn.join(&group, true, false)
    {
        Ok(_) => {},
        Err(e) => assert!(false, "Test failed: {}", e),
    }

    let conn2 = Connection::new(SockType::Rdm);
    assert_eq!(conn2.get_socket() != 0, true);

    match conn2.join(&group, false, false)
    {
        Ok(_) => {},
        Err(e) => assert!(false, "Test failed: {}", e),
    }

    // check for member join event
    let mut sock = IrisAddressType::new();
    let mut member = IrisAddressType::new();
    match conn.recv_from(&mut sock, &mut member)
    {
        Ok(buf) => {
            if buf.len() != 0
            {
                assert!(false, "Not a member event!");
            }
        },
        Err(e) => assert!(false, "Test failed: {}", e),
    };

    // send data to group
    let data = vec!('a' as u8, 'b' as u8, 'c' as u8);

    match conn2.sendto(&data, &group)
    {
        Ok(l) => assert_eq!(l, 3),
        Err(e) => assert!(false, "Test failed: {}", e),
    }

    match conn.recv()
    {
        Ok(r) => {
            assert!(r[0] == 'a' as u8);
            assert!(r[1] == 'b' as u8);
            assert!(r[2] == 'c' as u8);
        },
        Err(e) => assert!(false, "Test failed: {}", e),
    };

    match conn.leave()
    {
        Ok(_) => {},
        Err(e) => assert!(false, "Test failed: {}", e),
    }

    match conn2.leave()
    {
        Ok(_) => {},
        Err(e) => assert!(false, "Test failed: {}", e),
    }
}
