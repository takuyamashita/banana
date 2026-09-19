テスト専用の署名鍵(RSA 2048)。トークン検証のテストで、トークンに署名するためだけに使う。
どの環境の認証基盤にも登録していない。作り直すときは次のとおり。

```sh
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out test-signing-key.pem
openssl pkey -in test-signing-key.pem -pubout -out test-signing-key.pub.pem
```
