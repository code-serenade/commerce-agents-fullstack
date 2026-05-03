# repo 层说明

`repo` 是项目中的数据访问层。

它的职责很单一：

- 封装单表读写
- 隔离 `sea-orm`
- 把数据库模型转换成业务可用的 DTO

## 核心原则

1. 这里只做单表操作。
2. 这里只负责数据访问，不负责业务编排。
3. 这是业务代码里唯一允许直接依赖 `sea-orm` 的层。

## 为什么要单独拆一层

这样做的目的是避免：

- `service` 直接拼 ORM 查询
- `web-server` 直接依赖数据库模型
- AI 在多个地方复制同样的数据库逻辑

## 使用方式

其他层不应该直接接触 entity，而应该通过 `repo` 暴露出的 Service 和 DTO 使用数据。

推荐调用链路：

```text
web-server -> service -> repo -> database
```

## 这层通常包含什么

- `entity/`
  - 由脚本生成的 SeaORM 模型

- `table/<table_name>/dto.rs`
  - 单表 DTO

- `table/<table_name>/service.rs`
  - 单表 Service
