#!/usr/bin/env python3
"""
从 kajweb/dict 和 qwerty-learner 下载词库数据并转换为 Binglish 格式。
输出格式: [{"w": "word", "ph": "phonetic", "tr": "translation", "en": "sentence", "cn": "句子"}]

用法: python3 scripts/convert_dict.py
"""

import json
import os
import zipfile
import urllib.request
import tempfile

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "resources")

# qwerty-learner 词库 URL (直接 JSON，格式简单)
QWERTY_BASE = "https://raw.githubusercontent.com/RealKai42/qwerty-learner/master/public/dicts"
QWERTY_SOURCES = {
    "words_toefl.json": f"{QWERTY_BASE}/TOEFL_3_T.json",
    "words_ielts.json": f"{QWERTY_BASE}/IELTS_3_T.json",
    "words_gre.json": f"{QWERTY_BASE}/GRE_3_T.json",
    "words_kaoyan.json": f"{QWERTY_BASE}/KaoYan_3_T.json",
}


def download_json(url):
    print(f"  Downloading: {url}")
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read().decode("utf-8"))


def convert_qwerty_format(data):
    """
    qwerty-learner 格式:
    [{"name": "word", "usphone": "phonetic", "trans": ["n. xxx", "v. xxx"]}]
    """
    result = []
    for item in data:
        word = item.get("name", "")
        if not word:
            continue
        phonetic = item.get("usphone", "") or item.get("ukphone", "")
        trans_list = item.get("trans", [])
        trans = "；".join(trans_list) if isinstance(trans_list, list) else str(trans_list)
        result.append({
            "w": word,
            "ph": phonetic,
            "tr": trans,
            "en": "",
            "cn": "",
        })
    return result


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    for filename, url in QWERTY_SOURCES.items():
        output_path = os.path.join(OUTPUT_DIR, filename)
        print(f"\nProcessing {filename}...")
        try:
            data = download_json(url)
            converted = convert_qwerty_format(data)
            with open(output_path, "w", encoding="utf-8") as f:
                json.dump(converted, f, ensure_ascii=False, separators=(",", ":"))
            print(f"  Done: {len(converted)} words -> {filename}")
        except Exception as e:
            print(f"  Error: {e}")
            # 保留空数组以确保编译通过
            with open(output_path, "w", encoding="utf-8") as f:
                json.dump([], f)

    print("\nAll done!")
    print("词库文件位于:", OUTPUT_DIR)


if __name__ == "__main__":
    main()
