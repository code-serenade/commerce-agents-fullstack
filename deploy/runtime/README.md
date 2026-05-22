# Commerce Agents 离线安装包

## 安装

本安装脚本只面向 Linux 服务器。Windows 和 macOS 不使用 `install.sh`。

```bash
./install.sh
```

执行前请先确认服务器已经安装并启动 Docker，且已安装 Docker Compose plugin。

安装脚本会：

```text
1. 检查 Docker 和 Docker Compose plugin
2. 加载 commerce-agents-images.tar
3. 创建 .env
4. 启动 postgres
5. 运行 db-init，先执行 migration，再执行一次性种子初始化
6. 启动 backend、web、admin
```

## 访问

```text
Web:   http://服务器IP:8080
Admin: http://服务器IP:8081
```

## 配置

需要调整端口、auth/file 服务地址、root 账号或数据库数据目录时，修改 `.env` 后执行：

```bash
docker compose up -d
```

`db-init` 会在每次启动时先执行 `migration up`，所以新增表或表结构变更会自动补齐；账号、权限和 root 用户种子初始化仍然只执行一次。

数据库数据默认保存在安装目录下的 `./data/postgres`，可通过 `.env` 中的 `POSTGRES_DATA_DIR` 修改。
