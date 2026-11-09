const BASE62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const BASE: i64 = 62;

pub struct Url {
    pub id: i64,
    pub long_url: String,
    pub short_url: String,
}
pub trait ShortenService {
    fn encode(&mut self);
}
impl ShortenService for Url {
    fn encode(&mut self) {
        let mut id: i64 = self.id;
        if id == 0 {
            self.short_url = "0".to_string();
            return;
        }
        let mut digits: Vec<u8> = Vec::new();
        while id > 0 {
            let remainder = id % BASE;
            digits.push(BASE62[remainder as usize]);
            id /= BASE;
        }
        digits.reverse();
        self.short_url = String::from_utf8(digits).unwrap();
    }
}

#[cfg(test)] // only compiled when you run `cargo test`
mod tests {
    use super::*; // lets the tests see Url, ShortenService, etc. from this file

    fn encode(id: i64) -> String {
        let mut url = Url {
            id,
            long_url: String::new(),
            short_url: String::new(),
        };
        url.encode();
        url.short_url
    }

    #[test]
    fn encodes_single_digits() {
        assert_eq!(encode(1), "1");
        assert_eq!(encode(61), "Z");
    }

    #[test]
    fn encodes_alex_xu_example() {
        assert_eq!(encode(11157), "2TX");
    }

    #[test]
    fn encodes_zero() {
        assert_eq!(encode(0), "0");
    }
}
