use thiserror::Error;

/// メールアドレスの形になっていない
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("メールアドレスが不正です")]
pub struct InvalidEmail;

/// メールアドレス。ログインと連絡に使う。
///
/// 前後の空白を除き、小文字にそろえて扱う。大文字小文字の違いだけのメールアドレスは同じものとみなす
/// (どのコンテキストでもこの扱いにそろえる)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidEmail> {
        let value = value.into().trim().to_ascii_lowercase();
        let valid = value.len() <= 254
            && !value.contains(char::is_whitespace)
            && value.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty()
                    && !domain.contains('@')
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
            });
        if !valid {
            return Err(InvalidEmail);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_to_lowercase_without_spaces() {
        assert_eq!(Email::parse(" Foo@Example.COM ").unwrap().as_str(), "foo@example.com");
    }

    #[test]
    fn address_without_domain_is_rejected() {
        assert_eq!(Email::parse("foo@"), Err(InvalidEmail));
        assert_eq!(Email::parse("foo@localhost"), Err(InvalidEmail));
        assert_eq!(Email::parse("foo@example."), Err(InvalidEmail));
    }

    #[test]
    fn address_with_two_at_signs_or_spaces_is_rejected() {
        assert_eq!(Email::parse("a@b@example.com"), Err(InvalidEmail));
        assert_eq!(Email::parse("a b@example.com"), Err(InvalidEmail));
    }
}
