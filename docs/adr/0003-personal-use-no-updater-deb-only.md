# 0003 自用定位：无应用内更新机制，只打 deb

Ubuntu 版是个人自用项目，不做公开发布与分发渠道运营。由此两个边界是有意为之：

- 移除/禁用 Windows 版的更新检查（`update.rs`，其检查的是上游 JYH1878 的 GitHub Releases，对本构建无意义）；有新版本时手动重新打包安装。
- 打包只出 deb（x86_64），不出 AppImage / snap / Flatpak：AppImage 无自更新优势可享（且仍依赖系统 WebKitGTK），snap/Flatpak 沙箱会拦截 `~/.kimi-code` 读取与 Secret Service 访问。

若日后转为公开发布，此 ADR 作废，需重议更新机制（deb 无法自更新，可选 apt 仓库）与打包矩阵。
