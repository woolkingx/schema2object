# Release Process (自動化)

## 前置設定（僅需一次）

### 1. 設定 PyPI Token

1. 到 https://pypi.org/manage/account/token/ 建立 API token
2. 到 GitHub repo → Settings → Secrets → Actions
3. 新增 secret：`PYPI_API_TOKEN` = 你的 token

### 2. 確認檔案

- [ ] `pyproject.toml` 版本號已更新
- [ ] `README.md` 更新（如有變更）
- [ ] 測試全部通過：`python -m pytest tests/test_objecttree.py`

---

## 發布流程（全自動）

### 方式一：GitHub Release（推薦）

```bash
# 1. 建立 git tag
git tag v0.1.0
git push origin v0.1.0

# 2. 到 GitHub 建立 Release
# - 點擊 "Releases" → "Draft a new release"
# - 選擇剛才的 tag: v0.1.0
# - 標題：v0.1.0
# - 描述：列出變更內容
# - 點擊 "Publish release"

# 3. GitHub Actions 自動執行：
#    ✓ 跑測試（Python 3.9-3.13）
#    ✓ 打包
#    ✓ 上傳到 PyPI
```

### 方式二：手動觸發

到 GitHub → Actions → "Publish to PyPI" → "Run workflow"

---

## 版本號規則

遵循 [Semantic Versioning](https://semver.org/)：

- `0.1.0` → `0.1.1`：Bug 修復
- `0.1.0` → `0.2.0`：新增功能（向後相容）
- `0.1.0` → `1.0.0`：Breaking changes

---

## Checklist（發布前檢查）

```bash
# 1. 更新版本號
vim pyproject.toml  # version = "0.1.1"

# 2. 本地測試
cd /path/to/parent
python -m pytest tests/test_objecttree.py -v

# 3. Commit 變更
git add pyproject.toml
git commit -m "Bump version to 0.1.1"
git push

# 4. 建立 Release（GitHub 網頁操作）
# - 自動觸發 CI/CD
# - 5-10 分鐘後在 PyPI 上線
```

---

## 緊急回退

如果發布有問題：

```bash
# 1. Yank bad version (不刪除，但標記為不推薦)
pip install twine
twine upload --skip-existing dist/*  # 上傳修復版本

# 2. 到 PyPI 網頁標記舊版本為 "yanked"
```

---

## 維護建議

### 零維護模式（推薦）

- ✅ 啟用 GitHub Actions（自動測試）
- ✅ 啟用 Stale bot（自動關閉舊 issue）
- ✅ 在 README 寫明："Maintenance mode - accepting critical bug fixes only"

### 低維護模式

每季度檢查一次：
- Python 新版本測試（GitHub Actions 自動跑）
- 回覆 critical issues（security/data loss）
- 合併高品質 PR

### 完全放棄維護

在 README 頂部加上：
```markdown
> ⚠️ This project is no longer maintained. Consider using [alternatives].
```

---

## 預期維護時間

| 模式 | 年均時間 | 說明 |
|------|---------|------|
| **零維護** | 0-2 小時 | 只處理 security issues |
| **低維護** | 2-5 小時 | 季度檢查 + critical fixes |
| **積極維護** | 20+ 小時 | 新功能 + 社群支援 |

**建議：前兩年採用「低維護模式」，之後視情況轉「零維護」。**
