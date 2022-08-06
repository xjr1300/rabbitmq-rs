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

## RabbitMQとメッセージブローカー

`RabbitMQ`はメッセージブローカーである。メッセージブローカーはメッセージを受け取り、送信する。
`RabbitMQ`のメッセージは単なるバイナリデータであり、`RabbitMQ`はメッセージの内容を解釈しない。

`プロデューサー`はメッセージを送信（生産）して、`コンシューマー`がメッセージを処理（消費）する。

`エクスチェンジ`は、`プロデューサー`と`コンシューマー`の間でメッセージを仲介する。
`エクスチェンジ`は、`プロデューサー`から受信したメッセージを`キュー`に蓄積する。

`コンシューマー`は、`キュー`に接続して、キューイングされたメッセージを処理する。

`エクスチェンジ`は、メッセージが`コンシューマー`によって確実に処理されたか管理する。
`エクスチェンジ`は、メッセージが処理されていないと判断した場合は、そのメッセージを再キューイングする。

`RabbitMQ`は、以下の場合メッセージを再キューイングする。

* 肯定応答を受診する前にコンシューマーと切断されてチャネルが閉じられたとき
* コンシューマーが再キューイングを指定して`reject`や`nack`で応答したとき

## 1. Hello World

プロデューサーが、 `デフォルトエクスチェンジ `にルーティングキー`hello`を使用してメッセージを送信することで、`hello`キューに、メッセージがキューイングされる。

コンシューマーは、`hello`キューにキューイングされたメッセージを取得して処理する。
コンシューマーが処理に成功した場合は、`RabbitMQ`に肯定応答を返却して、`RabbitMQ`にメッセージが正常に処理したことを伝える。
これにより`RabbitMQ`はこのメッセージを削除する。

`デフォルトエクスチェンジ`は、`メッセージブローカー（RabbitMQ)`によってい事前に準備された、名前を持たない`ダイレクトエクスチェンジ`である。

`デフォルトエクスチェンジ`にメッセージを送信したとき、そのメッセージの`ルーティングキー`と同じ名前の`キュー`にメッセージがキューイングされる。

![Hello World](https://www.rabbitmq.com/img/tutorials/python-one.png)

```bash
# コンシューマーを起動
cargo run --package hello_world --bin consumer
# プロデューサーがメッセージを発行
cargo run --package hello_world --bin producer
```
コンシューマーは、`Ctrl + C`で強制終了する。

## 2. Work Queues

![Work Queues](https://www.rabbitmq.com/img/tutorials/python-two.png)

このチュートリアルにおいて、複数のワーカーに渡って、時間を消費するタスクを分配するワークキューを作成する。

タスクキューとしても知られるワークキューの背後にある主要な理念は、すぐにリソースが集中するタスクを実行して、それが完了するまで待つ必要を避けることである。
代わりに、後で実行されるようにタスクをスケジュールする。
タスクをメッセージとしてカプセル化して、キューに送信する。
バックグラウンドで実行されているワーカープロセスは、タスクをキューから取り出し、最終的にタスクを実行する。
多くのワーカーを実行しているとき、タスクはそれらの間で共有される。

この概念は、短いHTTPリクエストウィンドウの間で複雑なタスクを処理sくることが困難なWebアプリケーションで特に有効である。

### ラウンドロビン割り当て（Round-robin dispatching）

`RabbitMQ`は、複数の`コンシューマー`が存在する場合、それぞれのメッセージを順番に次の`コンシューマー`に送信する。
平均的にすべての`コンシューマー`は同じ数のメッセージを受け取る。
メッセージを分配するこの方法は、`ラウンドロビン`とよばれる。

### メッセージの完了通知（Message acknowledgement）

タスクの実行には数秒かかる場合がある。
1つの`コンシューマー`が長いタスクを開始して、部分的に実行して終了した場合、何が起こるか困惑するかもしれない。
現在のコードにおいて、一旦、`RabbitMQ`が`コンシューマー`にメッセージを配達したら、即座にそのメッセージに削除するために印をつける。
この場合、もしワーカーを停止した場合、ワーカーがちょうど処理していたメッセージを失うだろう。
この特別なワーカーに割り当てた、まだ処理されていないすべてのメッセージは失われるだろう。

1つもタスクを失うことを望んでいない。
もし、ワーカーが停止した場合、他のワーカーにタスクを配達したい。

メッセージを決して失わないことを保証するために、`RabbitMQ`は[メッセージ完了通知](https://www.rabbitmq.com/confirms.html)をサポートする。
`完了通知`は、`コンシューマー`によって、特定のメッセージを受信及び処理されて、`RabbitMQ`がそのメッセージを自由に削除できることを`RabbitMQ`に伝えるために返送される。

もし、`コンシューマー`が`完了通知`を送信せずに停止（その`チャネル`が閉じられた、`コンシューマー`との接続が閉じられた、また`TCP`接続が失われた）した場合、`RabbitMQ`はメッセージが完全に処理されなかったことを理解して、そのメッセージを再キューイングする。
もし、同時に他の`コンシューマー`がオンラインに存在した場合、`RabbitMQ`はすぐに他の`コンシューマー`にそのメッセージを再配達する。
これにより、ワーカーが時々停止しても、1つもメッセージを失わないことを保証できる。

タイムアウト（デフォルトで30分）は、`コンシューマー`の配達確認（完了通知）に適用される。
これは、配達確認（完了通知）しないバグのある（立ち往生している）`コンシューマー`を検出することを支援する。
[配達通知タイムアウト（Delivery Acknowledgement Timeout）](https://www.rabbitmq.com/tutorials/tutorial-two-python.html#:~:text=Delivery%20Acknowledgement%20Timeout)で説明された方法で、このタイムアウトを増やすことができる。

[手動メッセージ完了通知（Manual message acknowledgement）](https://www.rabbitmq.com/confirms.html)はデフォルトで有効になっている。
正式な`RabbitMQ`のチュートリアルの`Hello world`では、`auto_ack=True`フラグによって明示的にそれらを無効にしていた。

```python
def callback(ch, method, properties, body):
    print(" [x] Received %r" % body)

channel.basic_consume(queue='hello', on_message_callback=callback, auto_ack=True)
```

ここでは、このフラグを除いて、タスクを終了したら、ワーカーから適切な`完了通知`を送信する。

```python
def callback(ch, method, properties, body):
    print(" [x] Received %r" % body.decode())
    time.sleep(body.count(b'.') )
    print(" [x] Done")
    ch.basic_ack(delivery_tag = method.delivery_tag)

channel.basic_consume(queue='hello', on_message_callback=callback)
```

このコードを使用することで、メッセージを処理している間に`Ctrl + C`を使用してワーカーを停止しても、何も失われないことを保証できる。
ワーカーが停止した後で、すぐに全ての`完了通知`されていないメッセージが再配達される。

`完了通知`は配達を受け取った同じチャネルで送信されなければならない。
異なるチャネルを使用して`完了通知`を試みることは、チャネルレベルでプロトコル例外を引き起こす。
より学習するために[確認ガイド（doc guide on confirming）](https://www.rabbitmq.com/tutorials/tutorial-two-python.html#:~:text=doc%20guide%20on%20confirmations)を参照すること。

### 完了通知の忘却

よくある間違いは`basic_ack`をしないことである。
簡単なエラーであるが、結果は10代である。
クライアントが停止したとき、メッセージは再配信される（ランダムな再配信のように見える）が、`完了通知`のないメッセージを解放することができずに、`RabbitMQ`はより多くのメモリを消費する。

この種類のミスをデバッグするために、`messages_unacknowledged`フィールドをプリントする`rabbitmqctl`を使用できる。

```bash
sudo rabbitmqctl list_queues name messages_ready messages_unacknowledged
```

### メッセージの永続性（Message durability）

もし`コンシューマー`が停止しても、タスクを失わないことを保証する方法を学んだ。
しかし、もし`RabbitMQ`サーバーが停止した場合、タスクはいまだに失われる。

`RabbitMQ`が停止またはクラッシュしたとき、指示しない限りキュートメッセージは忘れられる。
メッセージを失わないことを保証するためには2つのことが要求される。
キュートメッセージの両方を永続として印する必要がある。

最初に、`RabbitMQ`ノードの再起動後もキューが存続することを確認する必要がある。
そうするために、キューを永続として宣言する必要がある。

```python
channel.queue_declare(queue="hello", durable=True)
```

このコマンド自体は正しいにも関わらず、この設定では動作しない。
それは、`hello`とよばれるキューを既に定義指定ことが理由である。
`RabbitMQ`は、異なるパラメーターで既存のキューの再定義を許可せず、そのように試みるプログラムにエラーを返却する。
しかし、簡易な回避策がある ー それは例えば`task_queue`など異なる名前でキューを定義することである。

```python
channel.queue_declare(queue="task_queue", durable=True)
```

この`queue_declare`の変更は`プロデューサー`と`コンシューマー`コードの両方で適用される必要がある。

この時点で、`task_queue`キューは、`RabbitMQ`が再起動しても失われない。
現在、`pika.spec.PERSISTENT_DELIVERY_MODE`を値として`delivery_mode`プロパティに提供することで、メッセージを持続として印する必要がある。

```python
channel.basic_publish(
  exchange="",
  body=message,
  properties=pika.BasicProperties(
    delivery_mode=pika.spec.PERSISTENT_DELIVERY_MODE,
  )
)
```

#### メッセージ永続の留意事項

メッセージを永続として印することは、メッセージが失われないことを完全に保証しない。
しかしながら、それは`RabbitMQ`にディスクにメッセージを保存することを指示しているが、`RabbitMQ`がメッセージを受け取り、そしてそれを保存していない短い時間がある。
また、`RabbitMQ`全てのメッセージに対して`fsync(2)`を実行しない ー それは単にキャッシュに保存して、実際にディスクに書き込まれていない。
永続性の保証は強固でないが、それは単純なタスクキュー以上である。
もし、強固な補償が必要であれば、[パブリッシャーコンファーム（publisher confirms）](https://www.rabbitmq.com/confirms.html)を使用できる。

### 公平な割り当て（Fair despatch）

割り当てが望んでいるように正確に動作しないことに気づいたかもしれない。
たとえば、2つのワーカーがある状況で、奇数番目に到着したすべてのメッセージが重たく、偶数番目に到着したすべてのメッセージが軽いとき、1つのワーカーは常に忙しく、そしてもう一方はほとんど作業をしない。
`RabbitMQ`はそれについて何も知らないし、また均等にメッセージを割り当てる。

これは、メッセージがキューに登録されたとき、`RabbitMQ`が単にメッセージを割り当てることが理由である。
`RabbitMQ`は、`コンシューマー`のために`未完了通知`メッセージの数を見ない。
`RabbitMQ`は、盲目的にすべてのn番目のメッセージをn番目の`コンシューマー`に割り当てるだけである。

![Fair dispatch](https://www.rabbitmq.com/img/tutorials/prefetch-count.png)

解決するために、`prefetch_count=1`設定で`Channel#basic_qos`チャネルメソッドを使用できる。
これは、同時に1つ以上のメッセージをワーカーに与えないように、`RabbitMQ`に指示するために、`basic.qos`プロトコルメソッドを使用する。
または、言い換えれば、ワーカーが処理している、または前のメッセージの`完了通知`していない間に、新しいメッセージをワーカーに割り当てないことである。
代わりに、`RabbitMQ`はメッセージを次の忙しくないワーカーに割り当てる。

```bash
channel.basic_qos(prefetch_count=1)
```

#### キューサイズに関する留意事項

もし、すべてのワーカーが忙しい場合、キューが埋め尽くされる可能性がある。
それを監視してワーカーを追加するか、[メッセージTTL](https://www.rabbitmq.com/ttl.html)を使用することを推奨する。


