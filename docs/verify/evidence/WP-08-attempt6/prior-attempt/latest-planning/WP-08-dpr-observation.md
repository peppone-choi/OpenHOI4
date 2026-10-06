# WP-08 DPR 관측 경계

확인일: 2026-10-07. 새 독립 verify5의 exact9531121은 CSS viewport가 그대로인 native CDP DPR 1↔2 변화에서 Chrome·Edge·Chromium preferred/forced GL 여섯 경로의 canvas buffer 비율이 이전 DPR에 머물렀다. 기본5초 관측 뒤 Reset으로만 갱신됐다. Firefox·WebKit의 동적 CDP 경로는 가용하지 않아 검증하지 않았다. 이 기록은 OS 모니터를 옮긴 실험이나 GPU·브라우저 설정 변경이 아니다.

아래 공식 표준을 직접 열어 source 동작과 대조했다.

| 표준 | 확인한 범위 |
|---|---|
| [CSSOM View](https://drafts.csswg.org/cssom-view/#dom-window-devicepixelratio) | devicePixelRatio와 [MediaQueryList change](https://drafts.csswg.org/cssom-view/#mediaquerylist) 이벤트의 관측 계약 |
| [Media Queries Level 4](https://drafts.csswg.org/mediaqueries-4/#resolution) | resolution은 CSS 단위에 대한 출력 픽셀 밀도이며 page zoom을 고려한다 |
| [Resize Observer](https://drafts.csswg.org/resize-observer/#resize-observer-interface) | 기본 관측은 CSS content-box다. device-pixel-content-box는 장치 픽셀 크기를 별도로 관측하는 선택이다 |

CSS 크기가 같으면 기본 content-box observer만으로 DPR 변화 처리를 보장할 수 없다는 것은 표준과 실제 실패를 함께 대조한 해석이다. 최종 신호 선택과 자원 수명은 구현 ADR에 기록한다. pixel-content-box의 브라우저 지원을 확인하지 않고 모든 엔진에서 가용하다고 주장하지 않는다. resolution query를 사용한다면 현재 DPR 기준 query의 변경·재등록·해제와 abort/dispose 경계를 실제 검사해야 한다.

953의 원본250ms resize probe에는 새 surface 전환 중 camera·box epoch가 달라지며 raw0/clear background가 관측됐다. 이를 원본 그대로 보존했다. 별도 ready/sameepoch 대조에서는 red62는 raw 정상/PNG 배경 실패를 유지했고, current953은 raw와 PNG 서버 RGB가 모두 통과했다. 고정 지연 기록을 소급 변경하거나 첫 프레임 전 buffer를 건강한 렌더러로 세지 않는다.

Linux953의 Chromium locator screenshot total30000ms timeout은 별개다. fonts loaded 뒤 element stability 대기에서 시간 제한에 도달한 원본만으로는 제품 레이아웃과 테스트 동기화·공유 권위 runtime·총예산을 구분할 수 없다. 픽셀 단언·timeout·브라우저 선택을 약하게 해 통과시키지 않고 실제 box/canvas/frame·단계 시간·원본 trace로 진단한다.
