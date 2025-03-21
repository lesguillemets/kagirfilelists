# kagirfilelists

## Usage

```bash
kagirfilelists [OPTIONS] [DIR]
```

### Options

- `-o, --output <FILE>` に CSV 形式で書き込む．与えなければデフォルトは標準出力
- `-f, --force`: 上書きを許容する
- `DIR` の下のファイルを読み込む．


## Output Format


| Column              |                                                  |
|---------------------|--------------------------------------------------|
| `rel_path`          | 与えた `DIR` からファイルへの相対パス            |
| `file_name`         | ファイル名        .                              |
| `size`              | ファイルサイズ (bytes)     .                     |
| `created`           | それぞれ対応する日時 (local time)                |
| `modified`          | それぞれ対応する日時 (local time)                |
| `accessed`          | それぞれ対応する日時 (local time)                |
| `sha256`            | ファイルの sha-256 hash                          |
| `parent_dir`        | 親ディレクトリの名前（あれば）．                 |
| `parent_parent`     | 親ディレクトリの親の名前（あれば）．             |
| `full_path`         | ファイルの絶対パス                               |
| `created_epoch`     | epoch からの秒数で表記したもの                   |
| `modified_epoch`    | epoch からの秒数で表記したもの                   |
| `accessed_epoch`    | epoch からの秒数で表記したもの                   |

- `parent` と `parent_parent` は，例えば `project_sakura/doc/readme.txt` が複数個所にバックアップ取られていた時に，意味のある比較をしやすいように設定したもの
- UTF-8で保存される（はずな）ので，普通に windows の excel で開くと文字化けするかも．インポートするのが手っ取り早いようだ．
