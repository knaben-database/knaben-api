use std::collections::HashSet;
use actix_web::{HttpRequest, HttpResponse};
use lazy_static::lazy_static;
use crate::{error_handling, utils};

lazy_static! {
    static ref TROLL_USER_AGENTS: HashSet<&'static str> = {
        let mut user_agents: HashSet<&str> = HashSet::new();
        // Left server at 100% CPU usage
        user_agents.insert("python-requests/2.32.5");
        user_agents
    };
}

pub enum BlockMethod {
    _Blank,
    Troll,
    ContactSupport,
    Forbidden,
    None
}

pub fn is_blocked(req: &HttpRequest) -> BlockMethod {
    let Some(_ip) = utils::get_ip_address(&req) else {
        return BlockMethod::ContactSupport;
    };
    let Some(user_agent) = req.headers().get("User-Agent").and_then(|ua| ua.to_str().ok()) else {
        return BlockMethod::Forbidden;
    };

    if TROLL_USER_AGENTS.contains(user_agent) {
        return BlockMethod::Troll;
    }

    BlockMethod::None
}

pub fn troll_response() -> Result<HttpResponse, error_handling::Error> {
    use serde_json::json;

    let data = json!(
            {"hits":[{"bytes":667838054,"cachedOrigin":"Sukebei.nyaa.si python scraper","category":"XXX / Hentai / Pictures","categoryId":[5004005,5004000,5000000],"date":"2026-08-23T11:44:28+00:00","details":"https://sukebei.nyaa.si//view/4691446","grabs":null,"hash":"FCA0DC83C03AEF5EBBA2F3AAAEB87A478B1CC314","id":"6f0bb39d83f491ba98820c028e819e94e072b328","lastSeen":"2026-08-23T11:47:35+00:00","link":null,"magnetUrl":"magnet:?xt=urn:btih:fca0dc83c03aef5ebba2f3aaaeb87a478b1cc314&dn=%5B%E3%81%8E%E3%82%93%E3%81%AA%E3%82%93%5D%20%E8%96%AC%E3%81%A7%E7%9C%A0%E3%82%89%E3%81%95%E3%82%8C%E3%81%A6%E7%8A%AF%E3%81%95%E3%82%8C%E3%82%8B%E8%A9%B1%20File%20No.1%20%5BAI%E7%94%9F%E6%88%90%5D&tr=http%3A%2F%2Fsukebei.tracker.wf%3A8888%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce","peers":4,"score":null,"seeders":0,"title":"[ぎんなん] 薬で眠らされて犯される話 File No.1 [AI生成]","tracker":"sukebei.nyaa.si","trackerId":"sukebeinyaasi","virusDetection":0.0},{"bytes":854841098,"cachedOrigin":"The Pirate Bay (proxy)","category":"XXX / ImageSet","categoryId":[5000000,5002000],"date":"2026-08-23T09:46:21+00:00","details":"https://knaben.xyz/thepiratebay/description.php?id=84205423","grabs":null,"hash":"9EF203ED0C7318174F45A92E4FF3170D84A5D555","id":"940bcb323c776bb2db9f1f9a1778abcaefb7bd5e","lastSeen":"2026-08-23T10:19:21+00:00","link":null,"magnetUrl":"magnet:?xt=urn:btih:9EF203ED0C7318174F45A92E4FF3170D84A5D555&dn=%5BKALA+AI+NSFW%5D+Ingrid+%28Taimanin%29&tr=http%3A%2F%2Fp4p.arenabg.com%3A1337%2Fannounce&tr=udp%3A%2F%2F47.ip-51-68-199.eu%3A6969%2Fannounce&tr=udp%3A%2F%2F9.rarbg.me%3A2780%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2710%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2730%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2920%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Fopentracker.i2p.rocks%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.cyberia.is%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.internetwarriors.net%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Ftracker.pirateparty.gr%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.tiny-vps.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce","peers":0,"score":null,"seeders":0,"title":"[KALA AI NSFW] Ingrid (Taimanin)","tracker":"The Pirate Bay","trackerId":"thepiratebay","virusDetection":0.17688860539136683},{"bytes":1664299827,"cachedOrigin":"The Pirate Bay (proxy)","category":"TV / HD","categoryId":[2000000,2001000],"date":"2026-08-23T09:46:00+00:00","details":"https://knaben.xyz/thepiratebay/description.php?id=84204686","grabs":3,"hash":"DCAC024294801AFE299184F43339540225572AFC","id":"bfbff25eeccf4e44e8741ce6cf830211977d73ff","lastSeen":"2026-08-23T10:41:20+00:00","link":null,"magnetUrl":"magnet:?xt=urn:btih:DCAC024294801AFE299184F43339540225572AFC&dn=Boxing+2026.08.22+Tiah+Mai+Ayton+vs.+Ellie+Hellewell+%5BWEB+720p%5D+%7BDAZN%7D&tr=http%3A%2F%2Fp4p.arenabg.com%3A1337%2Fannounce&tr=udp%3A%2F%2F47.ip-51-68-199.eu%3A6969%2Fannounce&tr=udp%3A%2F%2F9.rarbg.me%3A2780%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2710%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2730%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2920%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Fopentracker.i2p.rocks%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.cyberia.is%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.internetwarriors.net%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Ftracker.pirateparty.gr%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.tiny-vps.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce","peers":0,"score":null,"seeders":0,"title":"Boxing 2026.08.22 Tiah Mai Ayton vs. Ellie Hellewell [WEB 720p] {DAZN}","tracker":"The Pirate Bay","trackerId":"thepiratebay","virusDetection":0.17191700268564827},{"bytes":452045307,"cachedOrigin":"The Pirate Bay (proxy)","category":"Movies / HD","categoryId":[2000000,3001000],"date":"2026-08-23T09:45:00+00:00","details":"https://knaben.xyz/thepiratebay/description.php?id=84204685","grabs":2,"hash":"10A601905BD5ECB31D2834645FB83904FA238064","id":"ca3755196c73c878ae22df8d53ef5fa264048ee6","lastSeen":"2026-08-23T10:36:44+00:00","link":null,"magnetUrl":"magnet:?xt=urn:btih:10A601905BD5ECB31D2834645FB83904FA238064&dn=The+War+Game+1966+1080p+BluRay+HDR10+DD+2+0+x265-GeneMige&tr=http%3A%2F%2Fp4p.arenabg.com%3A1337%2Fannounce&tr=udp%3A%2F%2F47.ip-51-68-199.eu%3A6969%2Fannounce&tr=udp%3A%2F%2F9.rarbg.me%3A2780%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2710%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2730%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2920%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Fopentracker.i2p.rocks%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.cyberia.is%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.internetwarriors.net%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Ftracker.pirateparty.gr%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.tiny-vps.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce","peers":8,"score":null,"seeders":1,"title":"The War Game 1966 1080p BluRay HDR10 DD 2 0 x265-GeneMige","tracker":"The Pirate Bay","trackerId":"thepiratebay","virusDetection":0.1686030215752031}],"max_score":null,"total":{"relation":"gte","value":10000}}
        );

    Ok(HttpResponse::Ok().json(data))
}

pub async fn _is_rate_limited(_req: &HttpRequest) -> bool {
    todo!()
}