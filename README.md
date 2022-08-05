# RabbitMQ and Rust

## RabbitMQコンテナ

- rabbitmq:3.10.7-management
  - バージョン3.10.7
  - 管理プラグイン付き

### RabbitMQコンテナ起動

```bash
docker-compose up -d
```

### RabbitMQ管理ページの表示

- URL: http://localhost:15672/
- ユーザー名: guest
- パスワード: guest

## Tutorial-1

```bash
# コンシューマーを起動
cargo run --package tutorial-1 --bin consumer
# プロデューサーがメッセージを発行
cargo run --package tutorial-1 --bin producer
```
コンシューマーは、`Ctrl + C`で強制終了する。
