# ADR-0102 검토된 AI 이미지의 릴리스 메타데이터 검사

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-01 후속(P-06), REQ-LEG-02, OPEN-09 |

## 맥락

01 OPEN-09의 사용자 답변과 02 §14.4·§14.5는 출처·생성 도구·이용 조건을 기록하고 인간 배포 검토를 통과한 AI 생성 이미지에 한해 배포물 포함을 허용한다. 기존 `check_assets.py --release`는 `ai_generated=true`를 일괄 거부했다. 이 ADR은 사용자 정책을 바꾸지 않고 기록의 기술 형식과 검사를 정한다. 실제 이미지 생성·등록·개별 검토·공개 배포는 이번 작업 범위에 없다.

## 결정

기존 필드와 라이선스 허용 목록을 유지한다. `source`는 기존처럼 필수 출처 문자열이다. `--release`에서 `ai_generated=true`인 항목은 다음 두 하위 테이블을 모두 요구한다. 개발 모드의 기존 AI 허용 동작은 유지하며, 릴리스 모드의 미검토 AI 거부 테스트도 유지한다.

| 테이블 | 필드 | 검사 |
|---|---|---|
| `ai_provenance` | `tool` | 공백만으로 구성되지 않은 문자열. 생성 도구·모델·버전 식별을 기록한다 |
| `ai_provenance` | `usage_terms_path` | 저장소 내 정규화된 상대 POSIX 경로. 존재하는 비어 있지 않은 파일 |
| `ai_provenance` | `usage_terms_sha256` | 소문자 16진수 64자리. 이용 조건 파일의 SHA-256과 일치 |
| `distribution_review` | `status` | 정확히 문자열 `approved`. `pending`, `rejected`, boolean false, 다른 값은 실패 |
| `distribution_review` | `reviewer` | 공백만으로 구성되지 않은 문자열. 인간 검토자의 식별자 |
| `distribution_review` | `reviewed_at` | 문자열 `YYYY-MM-DDTHH:MM:SSZ` 또는 `YYYY-MM-DDTHH:MM:SS±HH:MM`. 실제 유효한 날짜·시각, 초·시간대 필수 |
| `distribution_review` | `evidence_path` | 저장소 내 정규화된 상대 POSIX 경로. 존재하는 비어 있지 않은 검토 증거 파일 |
| `distribution_review` | `evidence_sha256` | 소문자 16진수 64자리. 검토 증거 파일의 SHA-256과 일치 |
| `distribution_review` | `asset_sha256` | 소문자 16진수 64자리. 해당 에셋 파일의 SHA-256과 일치 |

시각은 TOML 자체 datetime 값 대신 문자열로 저장한다. 소수 초·시간대 없는 시각·윤초는 이 기술 형식에서 지원하지 않는다. 현재 시각이나 미래 여부를 검사하지 않는다. 생성 도구의 실제 존재·버전과 검토자의 실명은 문자열 검사만으로 인증하지 않는다.

조건·증거 경로는 절대 경로, URL, 역슬래시, `..`, 비정규 경로를 거부하고 심볼릭 링크의 실제 경로도 저장소 안에 있어야 한다. 읽기 실패·없는 파일·빈 파일·잘못된 필드 타입·누락·해시 불일치는 릴리스 실패다. 조건 및 증거 파일을 `assets/`, `data/packs/`, `client/public/` 등 기존 검사 범위에 두면 해당 파일도 기존 매니페스트 등록 규칙을 따라야 한다. 개발 문서인 `docs/`에 두어도 참조와 해시는 검사한다. 릴리스 패키징에서 조건·고지와 검토 증거를 보존하는 일은 WP-39 소유다.

AI 이미지의 경로 확장자는 `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.svg`, `.avif`, `.bmp`, `.tif`, `.tiff` 중 하나여야 한다. 확장자 검사는 파일 내용을 디코딩하거나 이미지 진위를 보증하지 않는다. AI 폰트·데이터 등으로 허용 범위를 확대하지 않는다. 기존 폰트의 OFL-1.1 강제 검사, 금지 라이선스, 미등록·누락 파일, 경로·boolean·문자열 검사는 모두 별도로 적용한다.

다음은 형식 설명용 예시다. 해시 자리표시는 검사에 통과하지 않으며 실제 검토 기록이 아니다.

```toml
[[asset]]
path = "assets/example.png"
author = "실제 제작자"
source = "실제 생성 출처와 참조 입력 기록"
license = "CC-BY-SA-4.0"
modified = false
ai_generated = true
notes = "실제 검토에서 확인한 제한·고지"

[asset.ai_provenance]
tool = "실제 생성 도구 / 모델 / 버전"
usage_terms_path = "docs/asset-reviews/example-terms.md"
usage_terms_sha256 = "<조건 파일의 실제 SHA-256>"

[asset.distribution_review]
status = "approved"
reviewer = "실제 인간 검토자"
reviewed_at = "2026-10-06T09:00:00+09:00"
evidence_path = "docs/asset-reviews/example-review.md"
evidence_sha256 = "<검토 증거의 실제 SHA-256>"
asset_sha256 = "<이미지의 실제 SHA-256>"
```

이용 조건 파일에는 공식 조건의 출처 URL·확인일·해당 계정/서비스 조건 및 보존한 내용을 기록한다. 검토 증거에는 대상 이미지, 생성 도구·입력 출처, 이용 조건 및 배포 라이선스·고지를 검토한 인간의 판단과 근거를 남긴다. 이 내용의 충실성과 권리 관계는 인간 배포 검토와 WP-39 공개 게이트에서 확인한다.

소프트웨어 검사는 **인간 배포 검토 기록의 존재·상태·파일 일치**만 확인한다. 법적 심사의 자동 완료, 검토자 신원 인증, 서명 검증, 저작권이나 서비스 이용 조건의 적법성 보증을 수행하지 않는다. 기록과 해시를 함께 변경할 권한이 있는 사람이 허위 기록을 작성하는 행위까지 자동 검출할 수 없다. 파일 변경 시 일치하는 새 검토 기록을 요구하며 실제 재검토 여부는 인간이 확인한다. `--release` 통과는 공개 배포·태그 승인이 아니다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| AI 이미지 일괄 거부 유지 | 단순함 | OPEN-09 조건부 허용을 반영하지 못함 |
| `reviewed=true` boolean만 요구 | 간단한 입력 | 검토자·시각·증거·검토 대상의 버전을 확인할 수 없음 |
| 외부 URL만 조건·증거로 기록 | 자료 접근이 쉬움 | 네트워크 상태와 원격 내용 변경에 따라 검사가 달라지고 보존된 검토 대상을 확인할 수 없음 |
| 원격 신원/전자 서명 서비스 도입 | 추가 인증 가능 | 현 요구사항과 범위를 넘고 새 의존성·외부 서비스가 필요함 |

## 결과와 영향

Python 표준 라이브러리만 사용한다. 게임 규칙·시뮬레이션·RNG·저장 포맷·에셋 실제 내용은 변경하지 않는다. 비-AI 기존 매니페스트의 형식과 검사 동작을 유지한다. 추가 메타데이터를 갖춘 AI 이미지에도 기존 허용 라이선스 정책을 적용한다. 테스트는 임시 디렉터리의 합성 바이트와 가상 도구·검토자만 사용하며 배포 에셋이나 인간 승인 증거로 취급하지 않는다.

2026-10-06 확인한 공식 기술 자료: [Python 3.11 hashlib.file_digest](https://docs.python.org/3.11/library/hashlib.html#hashlib.file_digest), [Python 3.11 datetime.fromisoformat](https://docs.python.org/3.11/library/datetime.html#datetime.datetime.fromisoformat). 파일을 바이너리로 열어 SHA-256을 계산하고, 명시한 좁은 시각 문자열 형식에 정규식을 적용한 뒤 실제 날짜·시각을 파싱한다. 생성 서비스별 이용 조건은 실제 에셋을 검토할 때 별도로 확인해야 하며 이번 ADR은 특정 서비스의 법적 조건을 평가하지 않는다.
