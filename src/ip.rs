use actix_web::{get, HttpResponse, Result as ActixResult};
use std::{
    io::Result as IoResult,
    net::{IpAddr, UdpSocket},
};

#[get("/ip")]
pub(crate) async fn handler() -> ActixResult<HttpResponse> {
    let ip = get_local_ip()?;
    info!("local ip: {}", ip);
    Ok(HttpResponse::Ok().json(ip))
}

fn get_local_ip() -> IoResult<IpAddr> {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            let _ = socket.connect("1.1.1.1:80");
            // ^-- ignore error, the socket still gets the local ip
            socket.local_addr()
        })
        .map(|addr| addr.ip())
}
