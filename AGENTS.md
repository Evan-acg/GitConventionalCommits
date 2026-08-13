# AGENTS.md

## Code Navigation

Use `ast-grep outline` before reading full source files when exploring code.

- Use `ast-grep outline <file>` to inspect a candidate file.
- Use `ast-grep outline <dir> --items exports` to find public entry points.

After finding candidate files, use `ast-grep outline` to decide which source range to read. If your agent supports skills, use the `ast-grep-outline` skill for more detailed guidance.

## 破坏性命令防呆（铁律）

以下命令**执行前必须向用户明确确认**，得到批准后才可运行；未获批准一律禁止：

- 递归删除：`Remove-Item -Recurse`、`rm -rf`、`rd /s`、`rmdir /s`
- 强制删除：`Remove-Item -Force`、`rm -f`、`del /f`、`erase`
- 全盘/根路径操作：`format`、`diskpart`、针对家目录/盘符根目录的递归删除
- git 破坏性操作：`git reset --hard`、`git clean -fdx`、`git push -f`/`--force`、`git branch -D`
- 系统级：`Reset-Computer`、`Restart-Computer`、`Stop-Computer`、`Clear-RecycleBin`
- 权限/属主变更：`Takeown /F`、`icacls <路径> /reset`

特别警示（PowerShell 自动变量陷阱）：

- `$HOME`、`$home`、`$env:USERPROFILE`、`$env:APPDATA` 等自动变量是**只读的**，赋值不会生效且变量保持原值。
- 禁止将上述自动变量直接作为删除目标参数（如 `Remove-Item $root,$home`），否则可能递归删除家目录。
- 删除目标必须是显式字面量路径，且路径须经 `Test-Path` 校验后再操作。

代码层强制（见 ~/.config/opencode/plugins/guard.ts）：

- 破坏性命令被插件硬拦截，**必须显式包含 `CONFIRM-DESTRUCTIVE` 标记**才会放行。
- 用户说"可以删"不算数，必须用户亲手在命令中写入该标记。
