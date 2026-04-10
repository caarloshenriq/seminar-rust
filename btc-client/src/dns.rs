use std::net::{SocketAddr, UdpSocket};

/// Query a DNS server at `server` for A records of `hostname`.
/// Returns a list of IPv4 addresses.
pub fn lookup_a(hostname: &str, server: SocketAddr) -> std::io::Result<Vec<std::net::Ipv4Addr>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(server)?;

    let query = build_query(hostname);
    socket.send(&query)?;

    let mut buf = [0u8; 512];
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    let len = socket.recv(&mut buf)?;

    parse_a_records(&buf[..len])
}

/// Build a minimal DNS query packet for A records.
fn build_query(hostname: &str) -> Vec<u8> {
    let mut pkt = Vec::new();

    // Header
    pkt.extend_from_slice(&[0x12, 0x34]); // transaction ID
    pkt.extend_from_slice(&[0x01, 0x00]); // flags: standard query, recursion desired
    pkt.extend_from_slice(&[0x00, 0x01]); // QDCOUNT = 1
    pkt.extend_from_slice(&[0x00, 0x00]); // ANCOUNT = 0
    pkt.extend_from_slice(&[0x00, 0x00]); // NSCOUNT = 0
    pkt.extend_from_slice(&[0x00, 0x00]); // ARCOUNT = 0

    // Question: encode hostname as DNS labels
    for label in hostname.split('.') {
        pkt.push(label.len() as u8);
        pkt.extend_from_slice(label.as_bytes());
    }
    pkt.push(0x00); // end of name

    pkt.extend_from_slice(&[0x00, 0x01]); // QTYPE  = A
    pkt.extend_from_slice(&[0x00, 0x01]); // QCLASS = IN

    pkt
}

/// Parse A records out of a raw DNS response packet.
fn parse_a_records(buf: &[u8]) -> std::io::Result<Vec<std::net::Ipv4Addr>> {
    if buf.len() < 12 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "response too short",
        ));
    }

    let ancount = u16::from_be_bytes([buf[6], buf[7]]) as usize;
    let mut pos = 12;

    // Skip over the question section
    pos = skip_name(buf, pos)?;
    pos += 4; // QTYPE + QCLASS

    let mut addrs = Vec::new();

    for _ in 0..ancount {
        pos = skip_name(buf, pos)?;

        if pos + 10 > buf.len() {
            break;
        }

        let rtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let rdlen = u16::from_be_bytes([buf[pos + 8], buf[pos + 9]]) as usize;
        pos += 10;

        if rtype == 1 && rdlen == 4 && pos + 4 <= buf.len() {
            // A record
            addrs.push(std::net::Ipv4Addr::new(
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ));
        }

        pos += rdlen;
    }

    Ok(addrs)
}

/// Skip a DNS name (handles pointer compression too).
fn skip_name(buf: &[u8], mut pos: usize) -> std::io::Result<usize> {
    loop {
        if pos >= buf.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "name out of bounds",
            ));
        }
        match buf[pos] {
            0 => return Ok(pos + 1),
            n if n & 0xC0 == 0xC0 => return Ok(pos + 2), // pointer
            n => pos += 1 + n as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dns_response(ips: &[[u8; 4]]) -> Vec<u8> {
        let mut pkt = Vec::new();

        // Header
        pkt.extend_from_slice(&[0x12, 0x34]); // transaction ID
        pkt.extend_from_slice(&[0x81, 0x80]); // flags: response, recursion available
        pkt.extend_from_slice(&[0x00, 0x01]); // QDCOUNT = 1
        let ancount = ips.len() as u16;
        pkt.extend_from_slice(&ancount.to_be_bytes()); // ANCOUNT
        pkt.extend_from_slice(&[0x00, 0x00]); // NSCOUNT
        pkt.extend_from_slice(&[0x00, 0x00]); // ARCOUNT

        // Question section (dnsseed.local)
        for label in "dnsseed.local".split('.') {
            pkt.push(label.len() as u8);
            pkt.extend_from_slice(label.as_bytes());
        }
        pkt.push(0x00);
        pkt.extend_from_slice(&[0x00, 0x01]); // QTYPE = A
        pkt.extend_from_slice(&[0x00, 0x01]); // QCLASS = IN

        // Answer section
        for ip in ips {
            pkt.extend_from_slice(&[0xC0, 0x0C]); // pointer to question name
            pkt.extend_from_slice(&[0x00, 0x01]); // TYPE = A
            pkt.extend_from_slice(&[0x00, 0x01]); // CLASS = IN
            pkt.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]); // TTL = 60
            pkt.extend_from_slice(&[0x00, 0x04]); // RDLENGTH = 4
            pkt.extend_from_slice(ip);
        }

        pkt
    }

    #[test]
    fn test_parse_single_a_record() {
        let pkt = make_dns_response(&[[1, 2, 3, 4]]);
        let addrs = parse_a_records(&pkt).unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0], std::net::Ipv4Addr::new(1, 2, 3, 4));
    }

    #[test]
    fn test_parse_multiple_a_records() {
        let pkt = make_dns_response(&[[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]]);
        let addrs = parse_a_records(&pkt).unwrap();
        assert_eq!(addrs.len(), 3);
        assert_eq!(addrs[1], std::net::Ipv4Addr::new(5, 6, 7, 8));
    }

    #[test]
    fn test_parse_empty_response() {
        let pkt = make_dns_response(&[]);
        let addrs = parse_a_records(&pkt).unwrap();
        assert_eq!(addrs.len(), 0);
    }

    #[test]
    fn test_parse_too_short() {
        let result = parse_a_records(&[0x00, 0x01]);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_contains_hostname_labels() {
        let query = build_query("dnsseed.local");
        let needle = b"\x07dnsseed";
        assert!(query.windows(needle.len()).any(|w| w == needle));
    }

    #[test]
    fn test_query_ends_with_zero() {
        let query = build_query("dnsseed.local");
        assert_eq!(
            &query[query.len() - 4..],
            &[0x00, 0x00, 0x01, 0x00, 0x01][1..]
        );
    }
}
