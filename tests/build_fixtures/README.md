# 构建测试用例（fixtures）

`tests/fixtures/` 下的测试用例已提交到 git，是所有测试的唯一真源。只有在新增测试场景或修复 fixture 生成脚本 bug 时，才需要重新生成。

## 依赖安装

```bash
pip install python-docx openpyxl python-pptx reportlab
```

## 一键重建所有 fixture

```bash
cd tests/build_fixtures
python3 make_docx.py
python3 make_docx_lists.py
python3 make_xlsx.py
python3 make_pptx.py
python3 make_csv.py
python3 make_ipynb.py
python3 make_html.py
python3 make_misc.py
```

## 新增测试用例的步骤

1. 在对应的 `make_*.py` 脚本中添加一个 `build_NN_描述性名字()` 函数。
2. 运行该脚本生成新 fixture。
3. 在 `tests/<format>.rs` 添加对应的 `#[test]`。
4. 更新 `docs/test-matrix/` 下面的测试矩阵说明文档。
5. 执行 `cargo test`，首次运行会生成快照（snapshot）。
6. 检查 `tests/snapshots/` 目录下新生成的 `.snap` 文件，如果内容正确，提交到仓库。

## 命名规范与注意事项

- 文件命名格式：`NN_描述性名字.ext`（如 `01_basic.docx`）。
- 每个 fixture 只测试一个核心概念，不要将多个功能堆在一个文件。
- 如需覆盖特殊结构（定制 namespace、错误/特殊 xml、边界场景），建议手写 XML，不要单纯依赖 python-docx/openpyxl 等库，它们对底层结构做了太多封装。
- 请在 `docs/test-matrix/<format>.md` 文件里说明每个 fixture 的设计目标、验证点和已知缺口。

## 查看 extract-text 对 fixture 的行为

```bash
extract-text tests/fixtures/docx/01_basic.docx
```

仅建议在排查 snapshot diff 时临时查看。契约判定以 `docs/ENGINEERING_DECISIONS.md` 为准，覆盖范围以 `docs/test-matrix/` 为主，最终断言输出以 `tests/snapshots/` 内快照为准。
