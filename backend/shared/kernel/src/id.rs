use thiserror::Error;

/// 番号(ID)が正の整数でない
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("番号は正の整数である必要があります")]
pub struct InvalidId;

/// 正の整数の番号(ID)の型を宣言する。
///
/// 宣言した型ごとに別の型になるので、給与明細番号と派遣社員番号の取り違えはコンパイルエラーになる。
/// `from_i64`(0 以下は [`InvalidId`])と `as_i64` を持つ。
///
/// ```
/// platform_kernel::positive_id! {
///     /// 給与明細番号
///     pub struct PayslipId;
/// }
///
/// assert_eq!(PayslipId::from_i64(1).unwrap().as_i64(), 1);
/// assert_eq!(PayslipId::from_i64(0), Err(platform_kernel::InvalidId));
/// ```
#[macro_export]
macro_rules! positive_id {
    ($(#[$meta:meta])* $vis:vis struct $name:ident;) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name(i64);

        impl $name {
            pub fn from_i64(value: i64) -> ::core::result::Result<Self, $crate::InvalidId> {
                if value <= 0 {
                    return ::core::result::Result::Err($crate::InvalidId);
                }
                ::core::result::Result::Ok(Self(value))
            }

            #[must_use]
            pub fn as_i64(&self) -> i64 {
                self.0
            }
        }
    };
}

/// まだ登録していないことを表す目印。
///
/// 番号(ID)が登録したときに初めて決まるものは、登録前には番号を持たない。
/// そうしたものを `Project<Id>` のように番号の型で引数化し、登録前は番号の位置にこの目印を入れる
/// (`Project<Unsaved>`)。番号を尋ねられるのは登録済みのものだけになる。
/// どのコンテキストでも同じ意味で使うので、ここに置く
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsaved;
