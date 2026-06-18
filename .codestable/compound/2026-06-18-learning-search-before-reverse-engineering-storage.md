---
doc_type: learning
track: knowledge
slug: search-before-reverse-engineering-storage
title: 逆向外部应用的本地存储格式前，先搜——别自己猜
component: workflow
tags: [workflow, integration, storage, reverse-engineering, search-first]
severity: medium
created: 2026-06-18
source: feature 2026-06-18-cursor-cli-support
---

# 逆向外部存储前先搜

## 这个模式在什么情境下最有价值

集成任何"读外部应用的本地数据"的功能时——比如扫 cursor 的 chat、读某个 CLI 的 session、解析第三方 app 的配置 db。你需要从对方的数据文件里拿某个字段（cwd、id、时间、配置值），但不知道它存在哪、什么格式。

典型场景：要拿 cursor 每个 chat 的工作目录。会自然地开始逆向——看目录结构、猜编码、读 sqlite schema、写解码逻辑。

## 不这样做会出什么问题

自己逆向猜存储格式，会陷入"试一种、碰壁、换一种"的循环。cursor 找 cwd 我试了四条路（反向解码→agent-transcripts→正向匹配→worker.log），每条都部分工作但有坑，耗了大量轮次：

- 反向解码有编码歧义，全错
- 各种锚点都要回到反解，绕不出去
- 多源合并覆盖不全

**而真正的答案（cursor 把 Workspace Path 注入 system prompt 存在 store.db）自己逆向极难想到**——这是 cursor 的内部设计，没有公开文档说"我们往 system prompt 里塞了 workspace 路径"。但搜一下（论坛/issue/直接 grep db 内容）马上能看到。

## 更好的做法：先搜

动手逆向前，按这个顺序：

1. **Web 搜**："cursor chat session storage cwd location"、"cursor store.db schema"——社区很可能已经摸清楚了，或官方文档有
2. **直接看数据内容**：`sqlite3 db "SELECT data FROM ..."` 然后肉眼/grep 找你要的字段——很多时候数据本身就是明文（JSON/文本），不需要猜格式。cursor 的 Workspace Path 就是一行明文 `Workspace Path: /path`
3. **看 app 的开源代码/issue**：VSCode 系应用很多有公开 issue 讨论存储格式
4. **最后才逆向**：上面都没有，再系统性逆向格式

判据：如果你要找的字段，应用自己肯定需要（cursor 跑 chat 时肯定知道自己的 workspace），那它一定存在某处可读的数据里——优先用"搜/直接看内容"定位，而不是从目录名/编码反推。

## 反例：什么时候自己逆向也对

- 应用是闭源且无社区资料、db 内容是二进制 protobuf/加密——搜不到也看不懂，只能逆向
- 数据是你自己写的格式——直接看代码

但对 cursor/claude/codex 这种主流 AI CLI，社区资料充分、存储是 jsonl/sqlite 明文，先搜几乎总是更快。

## 下次怎么应用

集成新 CLI/外部应用时，第一步不是 `find` + `cat` 猜结构，而是：
1. web 搜 "<app> session/storage location <你要的字段>"
2. 直接 grep/dbdump 看数据明文里有没有目标字段
3. 还没有再逆向

省下的时间是"试错轮次 × 每轮成本"。cursor 这个 case，先搜能省掉前面四条弯路。

## 相关文档

- 触发本 learning 的 feature: `.codestable/features/2026-06-18-cursor-cli-support/`
- 具体踩坑细节: [[cursor-resume-workspace-scoped]]
