# rust_tivitter

Tivitter simple API server implemented in Rust.

Tivitter is a SNS for people who have wet their pants.



## Requirements
```
curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh

rustc --version
cargo --version
```

## DB
```
docker compose up -d
```

## API
```
cargo run --bin init_db # create DB tables
cargo run --bin rust_tivitter # run API server
cargo test -- --test-threads=1 # run test
```

## Debug Commands
```
mysql -u root -h 127.0.0.1 --ssl-mode=DISABLED --get-server-public-key -P 53306  -p
```

```
curl -X POST -H "Content-Type: application/json" -d '{"name":"test123"}' http://localhost:8888/api/users # ユーザ新規作成挙動の確認
curl -X POST -H "Content-Type: application/json" -d '{"name":"test123"}' -c cookie.txt http://localhost:8888/api/sessions # ログイン挙動とCookieの保存
curl -X POST -H "Content-Type: application/json" -d '{"content":"some tweet"}' -b cookie.txt http://localhost:8888/api/user_tweets # Cookieを使用してメモ作成
curl -X POST -H "Content-Type: application/json" -d '{"name":"test123"}' -b cookie.txt http://localhost:8888/api/follow_relations # Cookieを使用してフォロー
curl -H "Content-Type: application/json" -b cookie.txt http://localhost:8888/api/pages/timeline # Cookieを使用してタイムライン取得
```
