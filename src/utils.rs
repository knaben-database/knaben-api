use std::net::IpAddr;
use actix_web::HttpRequest;

pub fn get_ip_address(req: &HttpRequest) -> Option<IpAddr> {
    // Prioritize real ip header first
    if let Some(ip) = req.connection_info().realip_remote_addr().and_then(|addr| addr.parse::<IpAddr>().ok()) {
        return Some(ip);
    }
    // Then do the direct ip otherwise
    if let Some(ip) = req.peer_addr() {
        return Some(ip.ip());
    }

    None
}