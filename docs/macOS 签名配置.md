# macOS 签名与公证配置

> 适用：`.github/workflows/release.yml` 的 `macos` job、`scripts/package-macos.sh`
>
> 目标：让 macOS 产物通过 **Developer ID 签名 + Apple 公证**，用户下载 dmg 后双击即可打开，无 Gatekeeper 拦截。
>
> 不配置也不会失败——`package-macos.sh` 在没有 `MACOS_SIGN_IDENTITY` 时会产出**未签名** dmg（用户需右键→打开）。本文件是「要正式签名」时的操作手册。

---

## 1. 需要配置的 Secrets

在 GitHub 仓库 **Settings → Secrets and variables → Actions → New repository secret** 添加：

| Secret | 说明 | 来源 |
|---|---|---|
| `MACOS_SIGN_IDENTITY` | 签名身份字符串，形如 `Developer ID Application: Your Name (AB12CD34EF)` | 步骤 2 |
| `MACOS_P12_B64` | 导出证书 `.p12` 的 base64 | 步骤 3 |
| `MACOS_P12_PASSWORD` | 导出 `.p12` 时设置的密码 | 步骤 3 |
| `APPLE_API_KEY_ID` | App Store Connect API Key 的 Key ID | 步骤 4 |
| `APPLE_API_ISSUER` | Issuer ID | 步骤 4 |
| `APPLE_API_KEY_B64` | API Key `.p8` 的 base64 | 步骤 4 |

> 全部配齐才会走「签名 + 公证」；缺任一则会跳过对应环节。证书密码等**只放 Secrets，切勿提交到仓库**。

---

## 2. 生成 Developer ID Application 证书

需要一台 Mac（用 Xcode 最简单）：

1. Xcode → **Settings → Accounts** → 选中你的 Apple ID/团队 → **Manage Certificates…** → 左下 `+` → **Developer ID Application**。
2. 打开「钥匙串访问 (Keychain Access)」→ 左侧「我的证书」找到刚生成的证书（展开应能看到配套私钥）→ 右键 → **导出…** → 存为 `cert.p12`，设置一个导出密码。
3. 在 Mac 终端查看签名身份字符串（复制整行，含括号里的 Team ID）：
   ```bash
   security find-identity -v -p codesigning
   # 输出示例：
   # 1) AB12... "Developer ID Application: Your Name (AB12CD34EF)"
   ```

> 也可以在 developer.apple.com → Certificates 手动建证书（需用「钥匙串访问 → 证书助理 → 从证书颁发机构请求证书」生成 CSR），流程更繁琐，推荐用 Xcode。

---

## 3. base64 编码证书

**在 Mac 上：**
```bash
base64 -i cert.p12 | pbcopy            # 得到 MACOS_P12_B64
```

**在 Windows（PowerShell）上：**
```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.p12")) | Set-Clipboard
```

把结果粘到 `MACOS_P12_B64`；`MACOS_P12_PASSWORD` 填第 2 步设置的导出密码。

---

## 4. 生成 App Store Connect API Key（公证用）

1. 登录 [appstoreconnect.apple.com](https://appstoreconnect.apple.com) → **Users and Access → Integrations（Keys）→ Team Keys** → `+`。
2. 起个名字，角色选 **Developer**（或更高）→ 生成。
3. **下载 `.p8`**（只能下载一次，务必保存），记下 **Key ID** 与 **Issuer ID**。

base64 编码 `.p8`：

**Mac：** `base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy`
**Windows：** `[Convert]::ToBase64String([IO.File]::ReadAllBytes("AuthKey_XXXXXXXXXX.p8")) | Set-Clipboard`

分别填入 `APPLE_API_KEY_ID`、`APPLE_API_ISSUER`、`APPLE_API_KEY_B64`。

> 公证凭据也可以用 Apple ID + App 专用密码，但 **API Key 最适合 CI**，本项目按此实现。

---

## 5. 触发与验证

1. 先手动跑一次：Actions → **Release** → **Run workflow**（`workflow_dispatch`，此时不会创建 Release，只验证构建）。
2. 确认 macos job 日志里出现 `codesign` 与 `notarytool submit ... status: Accepted`。
3. 正式发布：`git tag v0.1.0 && git push --tags`。
4. 下载 dmg 验证：
   ```bash
   xcrun stapler validate Vidly-0.1.0.dmg   # 通过 = 已公证并装订
   spctl -a -vvv -t install Vidly.app       # 应显示 accepted / source=Notarized Developer ID
   ```

---

## 6. 常见报错

| 报错 | 原因 |
|---|---|
| `The binary is not signed with a valid Developer ID certificate` | 漏签嵌套的 `Contents/MacOS/ffmpeg`、`ffprobe`。脚本已按「先签嵌套二进制，再签 .app」的正确顺序处理，勿用 `--deep` 一把梭后改动嵌套文件 |
| `The executable does not have the hardened runtime enabled` | 缺 `--options runtime`（脚本已含） |
| `has a code signature but it is not timestamped` | 缺 `--timestamp`（脚本已含） |
| `bundle format not recognized`（公证） | 公证对象必须是 dmg/zip，不能是 `.app` 目录（脚本先生成 dmg 再公证） |
| 签名身份字符串不匹配 | `MACOS_SIGN_IDENTITY` 必须与 `.p12` 里的证书完全一致（含 Team ID） |

---

## 7. 备注

- **架构**：`macos-14` runner 产出 **arm64** 包。若要同时支持 Intel，参考 `docs/开源打包构建.md` §7.4 做 `lipo` universal2（暂不必）。
- **替换内置 ffmpeg**：LGPL 允许用户替换 `Contents/MacOS/ffmpeg`，但替换后签名会失效，用户可自行重签（属正常）。
- **本地测试签名**：可在 Mac 上 `export MACOS_SIGN_IDENTITY=... && ./scripts/package-macos.sh 0.1.0` 单独验证脚本。
- 相关文件：`scripts/package-macos.sh`、`scripts/make-icns.sh`、`.github/workflows/release.yml`。
