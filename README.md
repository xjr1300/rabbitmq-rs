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

### 実行

ターミナルを複数起動して、それぞれのターミナルでワーカーを実行する。
その後、`プロデューサー`でメッセージを発行すると、メッセージが各ワーカーで処理される。

```bash
# ワーカーの起動
cargo run --package work_queues --bin worker
```

```bash
# プロデューサーの起動
cargo run --package work_queues --bin new_task
```

## 3. Publish / Subscribe

前のチュートリアルにおいて、ワークキューを作成した。
ワークキューの背後にある想定は、それぞれのタスクが正確に1つのワーカーに配達されることである。
このパートにおいて、全く異なることを実施する ー メッセージを複数の`コンシューマー`に配達する。
このパターンは`パブリッシュ/サブスクライブ`として知られている。

そのパターンを説明するために、単純なロギングシステムを構築する。
そのロギングシステムは2つのプログラムで構成される ー 最初のプログラムはログメッセージを発出して、2番目のプログラムはそれらを受け取ってプリントする。

ロギングシステムにおいて、受信プログラムを実行中の全てのコピーは、メッセージを得る。
1つの受信者を実行してディスクにログを直接送ることができる。そして同時に他の受信者を実行して画面でログを見つことができる。

本質的に、発行されたログメッセージは全ての受信者にブロードキャストされる。

### エクスチェンジ

チュートリアルの前のパートにおいて、メッセージをキューに送信したり、キューからメッセージを受信した。
ここで、`RabbitMQ`における完全なメッセージングモデルを紹介する。

前のチュートリアルでカバーしたことを簡単に振り返る。

* `プロデューサー`はメッセージを送信するユーザーアプリケーションである。
* `キュー`はメッセージを蓄積するバッファーである。
* `コンシューマー`はメッセージを受信するユーザーアプリケーションである。

`RabbitMQ`におけるメッセージングモデルの核となる理念は、`プロデューサー`は決して直接キューにメッセージを送信しない。
実際、`プロデューサー`はメッセージがキューに配達されたどうかさえ知らないことがよくある。

代わりに、`プロデューサー`はメッセージを`エクスチェンジ`にのみ送信できる。
`エクスチェンジ`はとても単純なモノである。
片方で、`エクスチェンジ`は`プロデューサー`からメッセージを受信して、もう一方でそれらをキューに押し入れる。
`エクスチェンジ`は受け取ったメッセージで何をするのか正確に知っている必要がある。
メッセージを特定のキューに追加するべきか?
メッセージを多くのキューに追加するべきか?
または、メッセージを破棄するべきか?
そのルールは、`エクスチェンジ`の種類によって定義される。

![エクスチェンジ](https://www.rabbitmq.com/img/tutorials/exchanges.png)

`direct`、`topic`、`headers`そして`fanout`など、いくつかの`エクスチェンジ`の種類がある。
最後の1つの`fanout`に着目する。
`ファンアウトエクスチェンジ`を作成して、その`エクスチェンジ`を`logs`と呼ぶ。

```python
channel.exchange_declare(exchange="logs", exchange_type="fanout")
```

`ファンアウトエクスチェンジ`はとても単純である。
おそらく名前から想像できるように、`ファンアウトエクスチェンジ`は、受信した全てのメッセージを、知っている全てのキューにブロードキャストする。
そして、それは正確にロガーにとって必要なものである。

#### エクスチェンジのリスト

サーバーでエクスチェンジをリストするために、これまでにない便利な`rabbitmqctl`を実行できる。

```bash
sudo rabbitmqctl list_exchanges
```

このリストにおいて、いくつかの`amq.*`エクスチェンジとデフォルト（名前のない）エクスチェンジがあるだろう。
これらはデフォルトで作成されたが、現時点でそれらを使用することはないだろう。

#### デフォルトエクスチェンジ

前のチュートリアルのパートにおいては、`エクスチェンジ`について何も知らなかったが、メッセージをキューに送信することができた。
それが可能だった理由は、空の文字列（""）によって識別した`デフォルトエクスチェンジ`を使用したからである。

前にどのようにメッセージを発行したか思い出しなさい。

```python
channel.basic_publish(exchange="", routing_key="hello", body=message)
```

`exchange`パラメータはエクスチェンジの名前である。
空の文字列はデフォルトエクスチェンジまたは名前のないエクスチェンジを指し示す。
もし`routing_key`が存在すれば、メッセージは`routing_key`によって指定された名前のキューに送信される。

これで、代わりに名前の付けられたエクスチェンジに発行できる。

```python
channel.basic_publish(exchange="logs", routing_key="", body=message)
```

### 一時的なキュー（Temporary queues）

前に覚えているかもしれないが、特定の名前を持つキューを使用した（`hello`と`task_queue`を覚えているだろうか?）。
キューに名前をつけることができることは重要である ー ワーカーに同じキューを示す必要があるからである。
プロデューサーとコンシューマー間で、キューを共有する必要があるとき、キューに名前を与えることが重要である。

しかし、ロガーにとってはそうでない。
全てのログメッセージの一部でなく全てのログメッセージを見る（聞く）ことを望んでいる。
また、古いものでなく現在流れているメッセージのみに関心がある。
それを解決するためには2つのことが必要である。

最初に、`RabbitMQ`に接続するときは、更新されたからのキューが必要である。
それをするために、ランダムな名前でキューを作成するか、サーバーにランダムなキューを選択させる。`queue_declare`に空の`queue`パラメータを`queue_declare`に提供することでこれができる。

```python
result = channel.queue_declare(queue="")
```

この時点で、`result.method.queue`はランダムなキューの名前を含んでいる。
例えば、それは`amq.gen-JzTY20BRgKO-HjmUJj0wLg`のように見えるだろう。

![ランダムな名前を持つキュー](https://www.rabbitmq.com/img/tutorials/python-three-overall.png)

2番目に、一旦、コンシューマーの接続が閉じられたら、キューは削除されなくてはならない。
そのために`exclusive`フラグがある。

```python
result = channel.queue_declare(queue="", exclusive=True)
```

[guide on queues](https://www.rabbitmq.com/queues.html)で、`exclusive`フラグや他のキューのプロパティについてより学ぶことができる。

### バインディング（Bindings）

![バインディング](https://www.rabbitmq.com/img/tutorials/bindings.png)

既にファンアウトエクスチェンジとキューを作成している。
ここで、キューにメッセージを送信することをエクスチェンジに指示する必要がある。
エクスチェンジとキューの間の関係をバインディングと呼ばれている。

```python
channel.queue_bind(exchange="log", queue=result.method.queue)
```

現時点から、`log`エクスチェンはキューにメッセージを追加する。

#### バインディングのリスト

想像する通り、存在するバインディングをリストできる。

```bash
sudo rabbitmqctl list-bindings
```

### 実行方法

`receive_logs`コマンドを以下の2つの方法で起動する。

```bash
cargo run --package publish_subscribe --bin receive_logs > logs_from_rabbit.log
```

```bash
cargo run --package publish_subscribe --bin receive_logs
```

`emit_log`コマンドを実行して、ログを発行する。

```bash
cargo run --package publish_subscribe --bin emit_log
cargo run --package publish_subscribe --bin emit_log spam grok egg
```

## 4. ルーティング（Routing）

[前のチュートリアル](https://www.rabbitmq.com/tutorials/tutorial-three-python.html)では、単純なロギングシステムを構築した。
多くの受信者にログメッセージをブロードキャストできた。

このチュートリアルでは、それに機能を追加するつもりである ー メッセージの部分集合だけを購読するようにするつもりである。
例えば、すべてのログをコンソールにプリントする間に、クリティカルエラーメッセージのみをログファイルに送信する（ディスクに保存するために）。

### バインディング（Binding）

前の例において、バインディングを作成している。
このようなコードを思い出すだろう。

``python
channel.queue_bind(exchange=exchange_name, queue=queue_name)
```

バインディングは、エクスチェンジとキューの間の関連である。
これは単に以下のように読める。
キューはこのエクスチェンジからきたメッセージに興味がある。

バインディングは追加の`routing_key`パラメーターを受け取ることができる。
`basic_publish`パラメーターとの混同を避けるために、`routing_key`を`binding key`と呼ぶ。
このようにキーを使用してバインディングを作成できる。

```python
channel.queue_bind(exchange=exchange_name, queue=queue_name, routing_key="black")
```

バインディングキーの意味はエクスチェンジの種類に依存する。
前に使用した`ファンアウトエクスチェンジ`は、単にバインディングキーの値を無視する。

### ダイレクトエクスチェンジ（Direct Exchange）

前のチュートリアルのロギングシステムは、すべてのコンシューマーにすべてのメッセージをブロードキャストした。
それらの重要度に基づいてメッセージをフィルタリングできるように拡張する必要がある。
例えば、受信したクリティカルエラーのみのログメッセージをディスクに書き込み、警告や情報ログメッセージでディスクスペースを浪費しないスクリプトが必要な場合である。

`ファンアウトエクスチェンジ`を使用したが、それは十分な柔軟性を与えてくれない ー それは無意識にブロードキャストするだけである。

代わりに`ダイレクトエクスチェンジ`を使用するつもりである。
`ダイレクトエクスチェンジ`の背後にあるルーティングアルゴリズムは単純である ー メッセージはメッセージが持つ`ルーティングキー`と正確に一致する`バインディングキー`を持つキューに送信される。

![ダイレクトエクスチェンジ](https://www.rabbitmq.com/img/tutorials/direct-exchange.png)

それを想像するために、以下の構成を考える。

* エクスチェンジからキューへの矢印についている文字列は、`ルーティングキー`を示す。

この構成において、2つのキューにバインドされた`ダイレクトエクスチェンジX`を確認できる。
最初のキューはバインディングキー`orange`にバインドされ、そして2つ目のキューは、バインディングキー`black`と`green`の2つのバインディングを持つ。

そのような構成において、エクスチェンジに発行された`orange`をルーティングキーに持つメッセージは、`キューQ1`に配送される。
ルーティングキーに`black`または`green`を持つメッセージは`Q2`に向かう。
その他すべてのメッセージは廃棄される。

### 複数バインディング（Multiple bindings）

複数のキューを同じバインディングキーでバインドすることは完全に合法である。
例において、バインディングキー`black`で`X`と`Q1間のバインディングを追加できる。
この場合、`ダイレクトエクスチェンジ`は`ファンアウト`のように振る舞い、マッチングした全てのキューにメッセージがブロードキャストする。
ルーティングキー`black`を持つメッセージは、`Q1`と`Q2`の両方に配送される。

### ログの送出（Emitting logs）

ロギングシステムにこのモデルを使用するつもりである。
`ファンアウト`の代わりに`ダイレクトエクスチェンジ`にメッセージを送信するつもりである。
`ルーティングキー`でログの重要度を提供するつもりである。
このようにすれば、受信スクリプトは受信したい重要度を選択できるようになる。
最初に、ログの送出に着目する。

```python
channel.exchange_declare(exchange="direct_log", exchange_type="direct")
```

そして、メッセージを送信する準備が整う。

```python
channel.basic_publish(exchange="direct_logs", routing_key=severity, body=message)
```

単純にするために、重要度に`info`、`warning`そして`error`のうち1つを想定する。

### 購読（Subscribing）

メッセージの受信は、1つの例外を除いて、前のチュートリアルのように動作する ー 興味のある重要度について新しいバインディングを作成する。

```python
result = channel.queue_declare(queue="", exclusive=True)
queue_name = result.method.queue

for severity in severities:
  channel.queue_bind(exchange="direct_log", queue=queue_name, routing_key=severity)
```

### ルーティングチュートリアルの全容

![ルーティングチュートリアルの全容](https://www.rabbitmq.com/img/tutorials/python-four.png)

### 実行方法

警告とエラーログを受信してファイルに出力するコンシューマーを起動する。

```bash
cargo run --package routing --bin receive_logs_direct warn error > logs_from_rabbit.log
```

情報、警告及びエラー（すべての）ログを受信して標準出力に出力するコンシューマーを起動する。

```bash
cargo run --package routing --bin receive_logs_direct info warn error
```

ログを送信する。

```bash
cargo run --package routing --bin emit_log_direct info "[info] Run. Run. Or it will explode."
cargo run --package routing --bin emit_log_direct warn "[warn] Run. Run. Or it will explode."
cargo run --package routing --bin emit_log_direct error "[error] Run. Run. Or it will explode."
```

## 5. トピック（Topics）

[前のチュートリアル](https://www.rabbitmq.com/tutorials/tutorial-four-python.html)において、ロギングシステムを改善した。
ダミーのブロードキャストのみが可能な`ファンアウトエクスチェンジ`を使用する代わりに、`ダイレクトエクスチェンジ`を使用して、ログを選択して受け取る能力を得た。

しかし、`ダイレクトエクスチェンジ`の使用はロギングシステムを改善したが、未だ制限がある ー 複数の条件に基づいて経路を設定できない。

ロギングシステムにおいて、重要性に基づくだけでなく、ログを発出した源に基づいて購読することを望むかもしれない。
重要性（info/warn/critic...)と設備(auth/cron/kern...)の両方に基づいて経路を設定する、Unixツールの`syslog`から来たこの概念を知っているかもしれない。

それは多くの柔軟性をもたらす ー `cron`から来たクリティカルエラーだけでなく、`kern`から来た全てのログを受信することを望むかもしれない。

ロギングシステムにそれを実装するために、より複雑な`トピックエクスチェンジ`について学ぶ必要がある。

### トピックエクスチェンジ（Topic exchange）

`トピックエクスチェンジ`に送信されたメッセージは、任意の`ルーティングキー（routing_key）`を持つことができない ー それはドット（.）で区切られた言葉（ワード）のリストでなくてはならない。
ワードはなんでも良いが、通常メッセージに接続されたなんらかの機能を指定する。
いくつか有効なルーティングキーの例は、`stock.usd.nyse`、`nyse.vmw`、`quick.orange.rabbit`である。
ルーティングキー内に、255バイトまでに制限された好みの多くのワードを含めることができる。

バインディングキーもまた、同様の広晟である必要がある。
`トピックエクスチェンジ`の背後にあるロジックは、`ダイレクトエクスチェンジ`に似ている ー 特定のルーティングキーを持った送信されたメッセージは、マッチしたバインディングキーに拘束されたすべてのキューに配送される。
しかしながら、バインディングキーには2つの重要で特別なケースがある。

* `*(star)`は、正確に1つのワードに代替できる。
* `#(hash)`は、0またはいくつかのワードに代替できる。

この簡単な説明を例で示す。

![トピックエクスチェンジ](https://www.rabbitmq.com/img/tutorials/python-five.png)

この例において、すべて動物を説明するメッセージを送信する。
メッセージは、3つのワード（2つのドット）で構成されたルーティングキーを持って送信される。
ルーティングキーの最初のワードは機敏さを、2番目のワードは色を、そして3番目のワードは種を述べている（`<celerity>.<color>.<species>）。

3つのバインディングを作成した。
`Q1`はバインディングキー`*.orange.*`に、`Q2`はバインディングキー`*.*.rabbit`と`lazy.#`に拘束されている。

これらのバインディングは、以下のように概説できる。

* `Q1`はオレンジ色のすべての動物に興味がある。
* `Q2`はすべてのウサギと全ての怠惰なすべての動物を受信したい。

`quick.orange.rabbit`がルーティングキーに設定されたメッセージは、両方のキューに配送される。
`lazy.orange.elephant`が設定されたメッセージもまた、それら両方に向かう。
一方で、`quick.orange.fox`は最初のキュー（`Q1`）のみに向かい、そして`lazy.brown.fox`は2番目のキュー（`Q2`）のみに向かう。
`lazy.pink.rabbit`は2つのバインドに一致するにもかかわらず、2番目のキューに1回だけ配送される。
`quick.brown.fox`はどのバインディングにも一致しないため、そのメッセージは破棄される。

契約を破って`orange`や`quick.orange.male.rabbit`のような、1つまたは4つのワードでメッセージを送信すると何が起こるのか？
これらのメッセージは、どのバインディングもマッチせずに、失われる。

一方で、`lazy.orange.male.rabbit`は4つのワードを持つにもかかわらず、最後のバインディングに一致する（`lazy.#`に一致する）ので、2番目のキューに配送される。

#### トピックエクスチェンジ（Topic exchange）

トピックエクスチェンジは強力で、他のエクスチェンジのように振る舞うことができる。

キューが`#(hash)`バインディングキーで拘束されているとき ー それはルーティングキーに関わらずにすべてメッセージを受け取る ー `ファンアウトエクスチェンジ`のように。

特別な文字`*(start)`と`#(hash)`がバインディングに使用されていないとき、`トピックエクスチェンジ`は単に`ダイレクトエクスチェンジ`のように振る舞う。￥

### 実装

ロギングシステムに`トピックエクスチェンジ`を使用するつもりである。
`<facility>.<severity>`の2つのワードを持つルーティングキーのログを想定して作業する。

そのコードはほとんど[前のチュートリアル](https://www.rabbitmq.com/tutorials/tutorial-four-python.html)と同じである。

### 実行方法

各ターミナルでコンシューマーを起動する。

```bash
cargo run --package topics --bin receive_logs_topic "#"
```

```bash
cargo run --package topics --bin receive_logs_topic "kern.*"
```

```bash
cargo run --package topics --bin receive_logs_topic "*.critical"
```

```bash
cargo run --package topics --bin receive_logs_topic "kern.*" "*.critical"
```

プロデューサーからログを送信する。

```bash
cargo run --package topics --bin emit_log_topic "kern.critical" "A critical kernel error"
```
