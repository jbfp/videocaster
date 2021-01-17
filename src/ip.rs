use anyhow::Error;
use rocket::response::Debug;
use rocket_contrib::json::Json;
use std::net::{IpAddr, UdpSocket};

#[get("/ip")]
pub(crate) fn handler() -> Result<Json<IpAddr>, Debug<Error>> {
    let ip = get_local_ip()?;
    info!("local ip: {}", ip);
    Ok(Json(ip))
}

fn get_local_ip() -> Result<IpAddr, Error> {
    let ip = UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            let _ = socket.connect("1.1.1.1:80");
            // ^-- ignore error, the socket still gets the local ip
            socket.local_addr()
        })
        .map(|addr| addr.ip())?;

    Ok(ip)
}
