use std::{str, thread, time::Duration};
use xcb::Xid;
use xcb::{x, Connection};

fn get_window_property(conn: &Connection, window: x::Window, property_name: &[u8], property_type: x::Atom) -> Option<String> {
    let atom_cookie = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name: property_name,
    });
    let atom = conn.wait_for_reply(atom_cookie).ok()?.atom();
    
    let prop_cookie = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: atom,
        r#type: property_type,
        long_offset: 0,
        long_length: 1024,
    });
    
    if let Ok(reply) = conn.wait_for_reply(prop_cookie) {
        if reply.value_len() > 0 {
            if let Ok(value) = str::from_utf8(reply.value()) {
                return Some(value.to_string());
            } else {
                return Some(format!("{:?}", reply.value()));
            }
        }
    }
    
    None
}

fn get_window_name(conn: &Connection, window: x::Window) -> Option<String> {
    let utf8_string = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name: b"UTF8_STRING",
    }).and_then(|cookie| conn.wait_for_reply(cookie)).map(|reply| reply.atom()).unwrap_or(x::ATOM_STRING);

    get_window_property(conn, window, b"_NET_WM_NAME", utf8_string)
        .or_else(|| get_window_property(conn, window, b"WM_NAME", x::ATOM_STRING))
        .or_else(|| get_window_property(conn, window, b"WM_CLASS", x::ATOM_STRING))
}

fn main() {
    let (conn, screen_num) = Connection::connect(None).expect("Failed to connect to X server");
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).expect("Failed to get screen");

    // Get all top-level windows
    let query_tree = conn.send_request(&x::QueryTree {
        window: screen.root(),
    });
    let query_tree = conn.wait_for_reply(query_tree).expect("Failed to query the window tree");

    for window in query_tree.children() {
        let window_id = window.resource_id();
        let name = get_window_name(&conn, *window).unwrap_or_else(|| "No name".to_string());
        println!("Window ID: 0x{:x}, Name: {}", window_id, name);
    }
}

