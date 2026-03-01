# protest — Rust OKX×Bybit 价差套利（单方向 MVP）

> 目的：构建一个 Rust 的跨交易所价差套利系统（永续合约为主），先做**单方向**策略：
>
> **OKX 买入（开多） + Bybit 卖出（开空）**
>
> MVP 阶段以“能稳定跑 + 信号与状态机闭环”为第一目标，真实下单放在后续阶段。

---

## PROJECT_META（给 GPT/监督系统看的固定元信息）

PROJECT_NAME: protest  
PROJECT_TYPE: rust_spread_arbitrage  
PROJECT_START: 2026-03-01  
EXCHANGES: OKX, BYBIT  
DIRECTION: OKX_LONG + BYBIT_SHORT (single_direction_only)  
SYMBOL: (暂定) BTC-USDT-SWAP / BYBIT_BTCUSDT_PERP  
DATA_SYNC_THRESHOLD_MS: 80  
ENTER_THRESHOLD_PCT: 0.5  
EXIT_THRESHOLD_PCT: 0.001 # 价格重叠的理想阈值（后续可调整）
MAX_HOLD_SECONDS: 120 # 保底退出（防止卡死）
MODE: paper_first_then_live

---

## 一、系统架构（核心思路）

系统由 4 个关键模块组成：

### 1) 行情输入层：okx_ws_task / bybit_ws_task

- 各自维持 WebSocket 连接（必须考虑断线重连/可靠性）
- 订阅指定交易对的 top-of-book（买一/卖一）或最小深度数据
- 将交易所原始消息解析为**统一的单边 Tick 结构体**，然后写入各自的 `watch` 通道
- 只保留“最新值”，不做堆积

#### 单边 Tick 统一结构体（6字段）

Tick = {
exchange, # 交易所名字（统一格式）
bid, # 买一价格
bid_qty, # 买一深度/数量
ask, # 卖一价格
ask_qty, # 卖一深度/数量
ts_ms # 时间戳（建议为本机接收时间，用于同步判断）
}

> 说明：OKX 合约“张数/面值”可能需要换算，MVP 可先固定一个 symbol 并在 config 中写死换算规则。

---

### 2) 决策层：core_ws_task（watch + select!）

- 使用 `watch + select!` 监听两边最新 Tick
- 任意一边更新都会触发一次计算（因为 watch 不堆积，只关心最新）
- 核心目标：保证两边数据“近似同步”再计算价差并产生信号

#### 同步规则（必须）

只有当：
abs(ts_okx - ts_bybit) < DATA_SYNC_THRESHOLD_MS
才允许进行 spread 计算与信号判断；否则标记为 `stale`，禁止交易。

#### 价差计算（单方向）

本项目只做单方向：

- OKX 开多（用 okx ask 作为买入参考）
- BYBIT 开空（用 bybit bid 作为卖出参考）

spread_raw = (bybit_bid - okx_ask) / mid_price
mid_price 可用 (okx_ask + bybit_bid)/2 或任一侧 mid

> 注意：spread 属于“组合快照”，不放进单边 Tick 内部，由 core 计算。

---

### 3) 状态机层：state_machine（防机关枪 + 管理生命周期）

为避免出现“价差持续存在导致机关枪开仓”，状态机必须保证互斥：

状态（最小集）：

- Idle：空仓，允许寻找入场
- Entering：开仓中（禁止再次开仓）
- Holding：双腿已建立对冲仓位，等待退出条件
- Exiting：平仓中（禁止开仓，执行退出）

核心规则：

- 进入 Entering 后不允许再开仓
- 任何时刻触发风控或异常 -> 强制进入 Exiting（或 Abort）

必须考虑的现实问题（后续进入真实交易时）：

- 半边成交 / 部分成交 / 拒单：一边成交后，另一边必须在限定时间内对冲，否则强制退出

---

### 4) 执行层：exec_task（真正下单/平仓/撤单）

exec_task 负责对接交易所 REST / 私有 WS 下单、撤单、查订单状态：

- bybit_open_long / bybit_open_short
- okx_open_long / okx_open_short
- bybit_close_all / okx_close_all

建议通信方式（工程化）：

- core/state_machine -> exec_task：通过 channel 发送 `OrderCmd`
- exec_task -> state_machine：通过 channel 回传 `ExecEvent`（filled/rejected/partial）

MVP 阶段可先做 paper（模拟成交、只记录 CSV），不接真实下单。

---

### 5) 风控层：auth_func / risk_manager（最高优先级）

- 监控两边保证金率、仓位、异常断流、最大持仓时间等
- 一旦触发（例如保证金 < 20% 或数据 stale 持续）：
  - 立即阻止新开仓
  - 强制进入 Exiting，要求两边平仓（或降仓）

---

## 二、里程碑计划（给 GPT 用来监督“是否拖延”）

> 每个里程碑都有 DoD（Definition of Done）。未达到 DoD 视为未完成。

### M1 — 行情链路稳定（DoD）

- [ ] OKX WS 能连接、订阅、持续输出 Tick（含自动重连）
- [ ] BYBIT WS 能连接、订阅、持续输出 Tick（含自动重连）
- [ ] Tick 六字段完整、格式统一（exchange/bid/bid_qty/ask/ask_qty/ts_ms）
- [ ] 日志能看到每秒至少 1 次两边报价更新

### M2 — core 同步 + 价差输出（DoD）

- [ ] watch + select! 监听两边 Tick（不堆积）
- [ ] 实现同步判断：abs(ts_okx-ts_bybit) < 80ms 才计算
- [ ] 输出：okx_ask、bybit_bid、ts_diff、spread、stale 标记
- [ ] 当 stale 时明确“不产生交易信号”

### M3 — 信号 + 状态机闭环（paper）（DoD）

- [ ] 状态机实现：Idle/Entering/Holding/Exiting
- [ ] 入场条件：同步 && spread > ENTER_THRESHOLD_PCT
- [ ] 退出条件：spread < EXIT_THRESHOLD_PCT 或 max_hold_time 超时 或 stale 持续
- [ ] “开仓中禁止重复开仓”生效（防机关枪）
- [ ] 输出 signals.csv（记录每次 Enter/Exit 信号 + 当时价格与 spread）

### M4 — 执行层 stub（DoD）

- [ ] core/state_machine 通过 channel 发送 OrderCmd（即便只是 mock）
- [ ] exec_task 回传 ExecEvent（mock filled/partial/reject）
- [ ] 状态机能根据事件推进到 Holding / Exiting 并回到 Idle

### M5 — 真实交易（小额/测试网）（DoD）

- [ ] 下单/撤单/查单链路打通（至少一边）
- [ ] 半边成交保护：一边成交后 X 秒内另一边必须对冲，否则 Abort
- [ ] 风控可触发强制退出（保证金/最大持仓/断流）

---

## 三、每日打卡区（强烈建议每天更新一行，GPT 用于监督）

TODAY: 2026-03-01  
DAY_N: 1  
CURRENT_MILESTONE: M1  
DONE_TODAY:

- BLOCKER:
- TOMORROW_MIN_STEP (<=30min):
-

---

## 四、运行方式（待补）

- cargo run
- 配置加载（config/env）
- 订阅的 symbol

---

## 五、注意事项（安全）

- 不要把真实 API KEY/SECRET 提交到仓库
- 建议使用 `.env`（本地）+ `.env.example`（提交模板）
- `.env` 必须写进 `.gitignore`
