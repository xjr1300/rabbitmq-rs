# RabbitMQ Tutorial by Rust

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

## 1. Hello World

プロデューサーが、 `デフォルトエクスチェンジ `にルーティングキー`hello`を使用してメッセージを送信することで、`hello`キューに、メッセージがキューイングされる。

コンシューマーは、`hello`キューにキューイングされたメッセージを取得して処理する。
コンシューマーが処理に成功した場合は、`RabbitMQ`に肯定応答を返却して、`RabbitMQ`にメッセージが正常に処理したことを伝える。
これにより`RabbitMQ`はこのメッセージを削除する。

`デフォルトエクスチェンジ`は、`メッセージブローカー（RabbitMQ)`によってい事前に準備された、名前を持たない`ダイレクトエクスチェンジ`である。

`デフォルトエクスチェンジ`にメッセージを送信したとき、そのメッセージの`ルーティングキー`と同じ名前の`キュー`にメッセージがキューイングされる。

`RabbitMQ`は、以下の場合メッセージを再キューイングする。

* 肯定応答を受診する前にコンシューマーと切断されてチャネルが閉じられたとき
* コンシューマーが再キューイングを指定して`reject`や`nack`で応答したとき

![Hello World](https://www.rabbitmq.com/img/tutorials/python-one.png)

```bash
# コンシューマーを起動
cargo run --package hello_world --bin consumer
# プロデューサーがメッセージを発行
cargo run --package hello_world --bin producer
```
コンシューマーは、`Ctrl + C`で強制終了する。
