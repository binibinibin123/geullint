# GeulLint 규칙 100개

> 이 파일은 공개 규칙 카탈로그에서 재현 가능하게 생성됩니다.

<a id="advanced.honorific.jeo-jasin"></a>
## `advanced.honorific.jeo-jasin` — 저 자신 지칭

주어가 ‘저’인 문맥에서 재귀 대명사 ‘저 자신’을 제안합니다.

- 분류: `advanced`
- 신뢰도: `low`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `editorial`
- 예: `제가 제 자신을` → `제가 저 자신을`

<a id="grammar.conjugation.doe-to-dwae"></a>
## `grammar.conjugation.doe-to-dwae` — Doe To Dwae

‘되서’는 ‘돼서’로 쓰는 것이 맞습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `되서` → `돼서`

<a id="grammar.conjugation.dwae-to-doe"></a>
## `grammar.conjugation.dwae-to-doe` — Dwae To Doe

‘돼면’은 ‘되면’으로 쓰는 것이 맞습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `돼면` → `되면`

<a id="grammar.ending.deun-choice"></a>
## `grammar.ending.deun-choice` — 선택의 ‘-든지’

선택을 나타내는 연결 어미 ‘-든지’를 ‘-던지’와 구별합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `커피던지 차던지` → `커피든지 차든지`

<a id="grammar.ending.euryeo"></a>
## `grammar.ending.euryeo` — Euryeo

‘-려고’를 사용하세요.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할려고` → `하려고`

<a id="grammar.ending.hal-ge"></a>
## `grammar.ending.hal-ge` — 할게 표기

약속이나 의지를 나타내는 종결 어미 ‘-ㄹ게’를 바르게 씁니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할께` → `할게`

<a id="grammar.negation.an-before-predicate"></a>
## `grammar.negation.an-before-predicate` — An Before Predicate

부정 부사 ‘안’을 사용하세요.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `않 간다` → `안 간다`

<a id="grammar.negation.ji-anh"></a>
## `grammar.negation.ji-anh` — Ji Anh

‘-지 않았다’처럼 보조 용언은 붙여 씁니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `지 안았다` → `지 않았다`

<a id="grammar.particle.comitative-allomorph"></a>
## `grammar.particle.comitative-allomorph` — 접속 조사 ‘과/와’

앞말의 받침에 맞춰 접속 조사 ‘과/와’를 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `책와` → `책과`

<a id="grammar.particle.duplicate"></a>
## `grammar.particle.duplicate` — Duplicate

조사가 중복된 것 같습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `를를` → `를`

<a id="grammar.particle.instrumental-allomorph"></a>
## `grammar.particle.instrumental-allomorph` — 부사격 조사 ‘으로/로’

앞말의 받침에 맞춰 부사격 조사 ‘으로/로’를 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `책로` → `책으로`

<a id="grammar.particle.object-allomorph"></a>
## `grammar.particle.object-allomorph` — 목적격 조사 ‘을/를’

앞말의 받침에 맞춰 목적격 조사 ‘을/를’을 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `책를` → `책을`

<a id="grammar.particle.subject-allomorph"></a>
## `grammar.particle.subject-allomorph` — 주격 조사 ‘이/가’

앞말의 받침에 맞춰 주격 조사 ‘이/가’를 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `나무이` → `나무가`

<a id="grammar.particle.topic-allomorph"></a>
## `grammar.particle.topic-allomorph` — 보조사 ‘은/는’

앞말의 받침에 맞춰 보조사 ‘은/는’을 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `책는` → `책은`

<a id="punctuation.duplicate.comma"></a>
## `punctuation.duplicate.comma` — 쉼표 중복

연속으로 잘못 입력된 쉼표를 하나로 줄입니다.

- 분류: `punctuation`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `,,` → `,`

<a id="punctuation.no-space-before-mark"></a>
## `punctuation.no-space-before-mark` — No Space Before Mark

문장 부호 앞에는 띄어쓰지 않습니다.

- 분류: `punctuation`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: ` .` → `.`

<a id="punctuation.space-after-comma"></a>
## `punctuation.space-after-comma` — 쉼표 뒤 띄어쓰기

쉼표 뒤에 이어지는 한국어 문장을 한 칸 띄웁니다.

- 분류: `punctuation`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `사과,배` → `사과, 배`

<a id="punctuation.space-after-sentence-mark"></a>
## `punctuation.space-after-sentence-mark` — 문장 부호 뒤 띄어쓰기

마침표·느낌표·물음표 뒤의 다음 문장을 한 칸 띄웁니다.

- 분류: `punctuation`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `끝났다.다음` → `끝났다. 다음`

<a id="repetition.adjacent-word"></a>
## `repetition.adjacent-word` — 인접 단어 반복

같은 단어가 바로 이어서 반복된 부분을 찾습니다.

- 분류: `repetition`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `문서를 문서를` → `문서를`

<a id="repetition.ending"></a>
## `repetition.ending` — Ending

어미가 반복된 것 같습니다.

- 분류: `repetition`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `했습니다습니다` → `했습니다`

<a id="spacing.compound.database"></a>
## `spacing.compound.database` — 데이터베이스 붙여쓰기

한 단어로 굳어진 ‘데이터베이스’를 붙여 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `데이터 베이스` → `데이터베이스`

<a id="spacing.dependent-noun.geot"></a>
## `spacing.dependent-noun.geot` — Geot

의존 명사 ‘것’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `좋을것 같다` → `좋을 것 같다`

<a id="spacing.dependent-noun.jeok"></a>
## `spacing.dependent-noun.jeok` — Jeok

의존 명사 ‘적’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `본적 있다` → `본 적 있다`

<a id="spacing.dependent-noun.jul"></a>
## `spacing.dependent-noun.jul` — Jul

의존 명사 ‘줄’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `알줄 안다` → `알 줄 안다`

<a id="spacing.dependent-noun.jung"></a>
## `spacing.dependent-noun.jung` — Jung

의존 명사 ‘중’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `하는중` → `하는 중`

<a id="spacing.dependent-noun.ppun"></a>
## `spacing.dependent-noun.ppun` — Ppun

의존 명사 ‘뿐’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `기다릴뿐이다` → `기다릴 뿐이다`

<a id="spacing.dependent-noun.su"></a>
## `spacing.dependent-noun.su` — Su

의존 명사 ‘수’는 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할수 있다` → `할 수 있다`

<a id="spacing.dependent-noun.ttae"></a>
## `spacing.dependent-noun.ttae` — Ttae

의존 명사 ‘때’는 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `만날때` → `만날 때`

<a id="spacing.fixed.ppunman-anira"></a>
## `spacing.fixed.ppunman-anira` — Ppunman Anira

`spacing.fixed.ppunman-anira` 한국어 검사 규칙입니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `뿐만아니라` → `뿐만 아니라`

<a id="spacing.fixed.su-bakke"></a>
## `spacing.fixed.su-bakke` — Su Bakke

‘수밖에’는 붙여 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할 수 밖에` → `할 수밖에`

<a id="spelling.adverb.i-hi"></a>
## `spelling.adverb.i-hi` — I Hi

`spelling.adverb.i-hi` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `깨끗히` → `깨끗이`

<a id="spelling.confusable.oraen-oraet"></a>
## `spelling.confusable.oraen-oraet` — Oraen Oraet

`spelling.confusable.oraen-oraet` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `오랫만에` → `오랜만에`

<a id="spelling.confusable.waen-il"></a>
## `spelling.confusable.waen-il` — 웬일 표기

뜻밖의 일을 나타내는 ‘웬일’을 바르게 씁니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `왠일` → `웬일`

<a id="spelling.confusable.wen-waen"></a>
## `spelling.confusable.wen-waen` — Wen Waen

`spelling.confusable.wen-waen` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `왠만` → `웬만`

<a id="spelling.conjugation.boe-bwae"></a>
## `spelling.conjugation.boe-bwae` — Boe Bwae

`spelling.conjugation.boe-bwae` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `뵈요` → `봬요`

<a id="spelling.conjugation.dwaet"></a>
## `spelling.conjugation.dwaet` — Dwaet

`spelling.conjugation.dwaet` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `됬` → `됐`

<a id="spelling.lexical.aedalpeuda"></a>
## `spelling.lexical.aedalpeuda` — 애달프다 표기

권장 표기: ‘애달프다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `애닯다` → `애달프다`

<a id="spelling.lexical.anseong-matchum"></a>
## `spelling.lexical.anseong-matchum` — Anseong Matchum

`spelling.lexical.anseong-matchum` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `안성마춤` → `안성맞춤`

<a id="spelling.lexical.chireotda"></a>
## `spelling.lexical.chireotda` — 치렀다 표기

권장 표기: ‘치렀다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `치뤘다` → `치렀다`

<a id="spelling.lexical.chojeom"></a>
## `spelling.lexical.chojeom` — Chojeom

`spelling.lexical.chojeom` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `촛점` → `초점`

<a id="spelling.lexical.daega"></a>
## `spelling.lexical.daega` — Daega

`spelling.lexical.daega` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `댓가` → `대가`

<a id="spelling.lexical.daesup"></a>
## `spelling.lexical.daesup` — 대숲 표기

권장 표기: ‘대숲’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `댓숲` → `대숲`

<a id="spelling.lexical.dakdalhada"></a>
## `spelling.lexical.dakdalhada` — 닦달하다 표기

권장 표기: ‘닦달하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `닥달하다` → `닦달하다`

<a id="spelling.lexical.damgatda"></a>
## `spelling.lexical.damgatda` — 담갔다 표기

권장 표기: ‘담갔다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `담궜다` → `담갔다`

<a id="spelling.lexical.deulyeodaboda"></a>
## `spelling.lexical.deulyeodaboda` — 들여다보다 표기

권장 표기: ‘들여다보다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `드려다보다` → `들여다보다`

<a id="spelling.lexical.dodaeche"></a>
## `spelling.lexical.dodaeche` — Dodaeche

`spelling.lexical.dodaeche` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `도데체` → `도대체`

<a id="spelling.lexical.dwichidakkeori"></a>
## `spelling.lexical.dwichidakkeori` — 뒤치다꺼리 표기

권장 표기: ‘뒤치다꺼리’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `뒤치닥거리` → `뒤치다꺼리`

<a id="spelling.lexical.dwikkumchi"></a>
## `spelling.lexical.dwikkumchi` — 뒤꿈치 표기

권장 표기: ‘뒤꿈치’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `뒷굼치` → `뒤꿈치`

<a id="spelling.lexical.eoieopda"></a>
## `spelling.lexical.eoieopda` — Eoieopda

`spelling.lexical.eoieopda` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `어의없` → `어이없`

<a id="spelling.lexical.eojjaetdeun"></a>
## `spelling.lexical.eojjaetdeun` — Eojjaetdeun

`spelling.lexical.eojjaetdeun` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `어쨋든` → `어쨌든`

<a id="spelling.lexical.eolmakeum"></a>
## `spelling.lexical.eolmakeum` — 얼마큼 표기

권장 표기: ‘얼마큼’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `얼만큼` → `얼마큼`

<a id="spelling.lexical.eure"></a>
## `spelling.lexical.eure` — 으레 표기

권장 표기: ‘으레’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `으례` → `으레`

<a id="spelling.lexical.gaekjjeokda"></a>
## `spelling.lexical.gaekjjeokda` — 객쩍다 표기

권장 표기: ‘객쩍다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `객적다` → `객쩍다`

<a id="spelling.lexical.gaesu"></a>
## `spelling.lexical.gaesu` — Gaesu

`spelling.lexical.gaesu` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `갯수` → `개수`

<a id="spelling.lexical.geokkuro"></a>
## `spelling.lexical.geokkuro` — Geokkuro

`spelling.lexical.geokkuro` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `꺼꾸로` → `거꾸로`

<a id="spelling.lexical.geondeurida"></a>
## `spelling.lexical.geondeurida` — 건드리다 표기

권장 표기: ‘건드리다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `건들이다` → `건드리다`

<a id="spelling.lexical.geumse"></a>
## `spelling.lexical.geumse` — Geumse

`spelling.lexical.geumse` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `금새` → `금세`

<a id="spelling.lexical.gop-ppaegi"></a>
## `spelling.lexical.gop-ppaegi` — Gop Ppaegi

`spelling.lexical.gop-ppaegi` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `곱배기` → `곱빼기`

<a id="spelling.lexical.gurenarut"></a>
## `spelling.lexical.gurenarut` — 구레나룻 표기

권장 표기: ‘구레나룻’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `구렛나루` → `구레나룻`

<a id="spelling.lexical.gwebyeon"></a>
## `spelling.lexical.gwebyeon` — 궤변 표기

권장 표기: ‘궤변’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `괴변` → `궤변`

<a id="spelling.lexical.gwittuim"></a>
## `spelling.lexical.gwittuim` — 귀띔 표기

권장 표기: ‘귀띔’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `귀뜸` → `귀띔`

<a id="spelling.lexical.haesseukhada"></a>
## `spelling.lexical.haesseukhada` — 해쓱하다 표기

권장 표기: ‘해쓱하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `핼쓱하다` → `해쓱하다`

<a id="spelling.lexical.hamatteomyeon"></a>
## `spelling.lexical.hamatteomyeon` — Hamatteomyeon

`spelling.lexical.hamatteomyeon` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `하마트면` → `하마터면`

<a id="spelling.lexical.heoguhan-nal"></a>
## `spelling.lexical.heoguhan-nal` — 허구한 날 표기

권장 표기: ‘허구한 날’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `허구헌날` → `허구한 날`

<a id="spelling.lexical.huihanhada"></a>
## `spelling.lexical.huihanhada` — Huihanhada

`spelling.lexical.huihanhada` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `희안` → `희한`

<a id="spelling.lexical.hyeolhyeoldansin"></a>
## `spelling.lexical.hyeolhyeoldansin` — 혈혈단신 표기

권장 표기: ‘혈혈단신’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `홀홀단신` → `혈혈단신`

<a id="spelling.lexical.iljjiki"></a>
## `spelling.lexical.iljjiki` — 일찍이 표기

권장 표기: ‘일찍이’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `일찌기` → `일찍이`

<a id="spelling.lexical.jamgatda"></a>
## `spelling.lexical.jamgatda` — 잠갔다 표기

권장 표기: ‘잠갔다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `잠궜다` → `잠갔다`

<a id="spelling.lexical.jjagipgi"></a>
## `spelling.lexical.jjagipgi` — Jjagipgi

`spelling.lexical.jjagipgi` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `짜집기` → `짜깁기`

<a id="spelling.lexical.jjalmakhada"></a>
## `spelling.lexical.jjalmakhada` — 짤막하다 표기

권장 표기: ‘짤막하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `짧막하다` → `짤막하다`

<a id="spelling.lexical.jjejjehada"></a>
## `spelling.lexical.jjejjehada` — 쩨쩨하다 표기

권장 표기: ‘쩨쩨하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `째째하다` → `쩨쩨하다`

<a id="spelling.lexical.jjigae"></a>
## `spelling.lexical.jjigae` — Jjigae

`spelling.lexical.jjigae` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `찌게` → `찌개`

<a id="spelling.lexical.kkeopjiljjae"></a>
## `spelling.lexical.kkeopjiljjae` — 껍질째 표기

권장 표기: ‘껍질째’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `껍질채` → `껍질째`

<a id="spelling.lexical.kkeorimchikhada"></a>
## `spelling.lexical.kkeorimchikhada` — 꺼림칙하다 표기

권장 표기: ‘꺼림칙하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `꺼림직하다` → `꺼림칙하다`

<a id="spelling.lexical.matbogi"></a>
## `spelling.lexical.matbogi` — 맛보기 표기

권장 표기: ‘맛보기’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `맛배기` → `맛보기`

<a id="spelling.lexical.mirunamu"></a>
## `spelling.lexical.mirunamu` — 미루나무 표기

권장 표기: ‘미루나무’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `미류나무` → `미루나무`

<a id="spelling.lexical.myeochil"></a>
## `spelling.lexical.myeochil` — 며칠 표기

‘몇일’을 표준어 ‘며칠’로 고칩니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `몇일` → `며칠`

<a id="spelling.lexical.naerorahada"></a>
## `spelling.lexical.naerorahada` — 내로라하다 표기

권장 표기: ‘내로라하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `내노라하다` → `내로라하다`

<a id="spelling.lexical.napjakhada"></a>
## `spelling.lexical.napjakhada` — 납작하다 표기

권장 표기: ‘납작하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `납짝하다` → `납작하다`

<a id="spelling.lexical.neolbjeokhada"></a>
## `spelling.lexical.neolbjeokhada` — 넓적하다 표기

권장 표기: ‘넓적하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `넙적하다` → `넓적하다`

<a id="spelling.lexical.neoljjikhada"></a>
## `spelling.lexical.neoljjikhada` — 널찍하다 표기

권장 표기: ‘널찍하다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `넓직하다` → `널찍하다`

<a id="spelling.lexical.omeurida"></a>
## `spelling.lexical.omeurida` — 오므리다 표기

권장 표기: ‘오므리다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `오무리다` → `오므리다`

<a id="spelling.lexical.osundosun"></a>
## `spelling.lexical.osundosun` — 오순도순 표기

권장 표기: ‘오순도순’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `오손도손` → `오순도순`

<a id="spelling.lexical.putnaegi"></a>
## `spelling.lexical.putnaegi` — 풋내기 표기

권장 표기: ‘풋내기’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `풋나기` → `풋내기`

<a id="spelling.lexical.seolgeoji"></a>
## `spelling.lexical.seolgeoji` — 설거지 표기

권장 표기: ‘설거지’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `설겆이` → `설거지`

<a id="spelling.lexical.seungnak"></a>
## `spelling.lexical.seungnak` — 승낙 표기

권장 표기: ‘승낙’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `승락` → `승낙`

<a id="spelling.lexical.silhjeungi-nada"></a>
## `spelling.lexical.silhjeungi-nada` — 싫증이 나다 표기

권장 표기: ‘싫증이 나다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `실증이 나다` → `싫증이 나다`

<a id="spelling.lexical.sutgarak"></a>
## `spelling.lexical.sutgarak` — 숟가락 표기

권장 표기: ‘숟가락’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `숫가락` → `숟가락`

<a id="spelling.lexical.tongjjaero"></a>
## `spelling.lexical.tongjjaero` — Tongjjaero

`spelling.lexical.tongjjaero` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `통채로` → `통째로`

<a id="spelling.lexical.ukyeoneohda"></a>
## `spelling.lexical.ukyeoneohda` — 욱여넣다 표기

권장 표기: ‘욱여넣다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `우겨넣다` → `욱여넣다`

<a id="spelling.lexical.umcheurida"></a>
## `spelling.lexical.umcheurida` — 움츠리다 표기

권장 표기: ‘움츠리다’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `움추리다` → `움츠리다`

<a id="spelling.lexical.umkeum"></a>
## `spelling.lexical.umkeum` — 움큼 표기

권장 표기: ‘움큼’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `웅큼` → `움큼`

<a id="spelling.lexical.uteoreun"></a>
## `spelling.lexical.uteoreun` — 웃어른 표기

권장 표기: ‘웃어른’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `윗어른` → `웃어른`

<a id="spelling.lexical.yeokhal"></a>
## `spelling.lexical.yeokhal` — Yeokhal

`spelling.lexical.yeokhal` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `역활` → `역할`

<a id="spelling.lexical.yosae"></a>
## `spelling.lexical.yosae` — 요새 표기

권장 표기: ‘요새’

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `요세` → `요새`

<a id="spelling.lexical.yukgaejang"></a>
## `spelling.lexical.yukgaejang` — Yukgaejang

`spelling.lexical.yukgaejang` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `육계장` → `육개장`

<a id="spelling.loanword.curated"></a>
## `spelling.loanword.curated` — Curated

`spelling.loanword.curated` 한국어 검사 규칙입니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `메세지` → `메시지`

<a id="style.redundancy.gajang-choego"></a>
## `style.redundancy.gajang-choego` — Gajang Choego

`style.redundancy.gajang-choego` 한국어 검사 규칙입니다.

- 분류: `style`
- 신뢰도: `high`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `editorial`
- 예: `가장 최고` → `최고`

<a id="style.redundancy.majority-over"></a>
## `style.redundancy.majority-over` — 과반수 이상 중복

‘과반수’에 이미 절반을 넘는다는 뜻이 있어 ‘이상’이 겹칠 수 있음을 알립니다.

- 분류: `style`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `editorial`
- 예: `과반수 이상` → `과반수`

<a id="technical.term.web-browser"></a>
## `technical.term.web-browser` — 웹 브라우저 표기

기술 용어 ‘웹 브라우저’의 철자를 바로잡습니다.

- 분류: `technical`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `웹부라우저` → `웹 브라우저`
