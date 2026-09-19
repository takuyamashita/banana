// sqlx::migrate! は、既にあるマイグレーションの中身の変更は追跡するが、ファイルが増えたことは追跡しない。
// migrations/ を見張って、マイグレーションを足したら埋め込み直す(sqlx migrate build-script と同じ)
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
