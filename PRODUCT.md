# Product

## Register

product

## Users

Fenestra 社区的 Rust 开发者与 GUI 技术选型者。他们在评估"纯 Rust 原生 GUI 能做到什么视觉上限"时打开这个应用：可能是在技术分享会上投屏演示，可能是在 GitHub 上 clone 后本地跑起来把玩，也可能是在对比 Tauri/Electron/Dioxus 时打开这个参照物。使用场景多为短时间高专注的审视：三分钟内决定"这套栈的观感是否令人信服"。

## Product Purpose

一个 WeChat 风格桌面客户端的完整参考实现：三栏布局（导航/会话/聊天）、真实 Rust 存储引擎（FTS 索引、LZ4 压缩模拟、WAL）、增量同步引擎（seq 序号、outbox、网络条件模拟）、诊断面板、基准测试与视觉特效画廊。它存在的意义是证明：fenestra 的确定性渲染管线可以承载一个"看起来像旗舰产品"的完整应用。成功的标准：开发者看完产生"我想用这套栈写我的下一个工具"的冲动。

## Brand Personality

有机光感（organic light）。三个词：**alive、luminous、precise**。表面有生命感（呼吸、漂移、光晕），像 Arc 浏览器和 visionOS 的玻璃景深，但每一处发光都有工程依据，OKLCH token 取色、确定性渲染、golden-lock 验证。GPU candy, with taste.

## Anti-references

- **赛博朋克霓虹堆砌**：满屏发光描边、RGB 电竞风、无意义 bloom。光来自"场景里有光源"，不来自 CSS glow。
- **玻璃滥用**：所有面板都毛玻璃，可读性让位于风格。玻璃是稀有材料，一次只用在一处。
- **平庸 Electron 风**：扁平卡片、默认蓝、零个性的通用聊天 UI。这是这个项目存在所要反驳的东西。
- **同质卡片网格**：所有指标装进一样的边框盒子。指标用字阶和留白建立层级，不靠卡片边框。

## Design Principles

1. **渲染即简历（The render is the résumé）**：每个视觉选择都在展示 fenestra 独有的能力，OKLCH token 光谱、两遍玻璃景深、确定性 headless 验证。如果一个效果用 CSS 也能做，它没有说服力。
2. **活而不闹（Alive, not loud）**：生命感来自持续的低幅度有机运动（呼吸、漂移、光晕脉动），从不来自突兀的爆发式动画。盯着看 10 秒，应该察觉到表面在呼吸。
3. **光有来源（Light has a source）**：每一处发光都能指出光源在哪，极光幕布在背景，镜面边缘光在玻璃上，状态光点在指示器上。无源之光（纯装饰 glow）禁止。
4. **冲击力靠语言不靠堆料（Impact via language, not stacking）**：视觉冲击来自一套贯穿全应用的光感语言（极光洗底 + 单一玻璃焦点 + 活体指示），而非效果数量的叠加。消息流永远保持可读性底线。
5. **优雅降级是功能（Graceful degradation is a feature）**：reduced motion 下所有呼吸/入场钉住首帧，玻璃退化为纯 tint——降级后的静态画面依然完整成立。

## Accessibility & Inclusion

标准底线：reduced motion 自动降级（fenestra 内建，钉住关键帧首帧）；APCA 对比度下限由主题系统保证（`validate_contrast`）；键盘可达（Tab 循环 + 焦点环）。演示场景不要求 WCAG 全量合规，但对比度与动效降级不可妥协。
