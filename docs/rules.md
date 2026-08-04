# GeulLint 규칙 116개

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
## `grammar.conjugation.doe-to-dwae` — ‘되’와 ‘돼’ 활용 구별

‘되서’, ‘되요’와 잘못 줄인 ‘됀’, ‘됄’, ‘됌’을 올바른 활용으로 고칩니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `됀` → `된`

<a id="grammar.conjugation.dwae-to-doe"></a>
## `grammar.conjugation.dwae-to-doe` — ‘돼’와 ‘되’ 활용 구별

‘돼게’, ‘돼면서’, ‘돼도록’처럼 어미 앞에서는 ‘되’를 씁니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `돼게` → `되게`

<a id="grammar.copula.anieyo"></a>
## `grammar.copula.anieyo` — ‘아니에요’ 표기

‘아니다’에 ‘-에요’가 붙은 활용형을 ‘아니에요’로 바로잡습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `아니예요` → `아니에요`

<a id="grammar.ending.colloquial-yong"></a>
## `grammar.ending.colloquial-yong` — 표준 종결어미 ‘-요’

편집 문체에서 ‘해용’, ‘세용’을 표준 종결어미로 검토하도록 안내합니다.

- 분류: `grammar`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `editorial`
- 예: `감사해용` → `감사해요`

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
## `grammar.ending.euryeo` — 의도·조건의 ‘-려고/-려면’

검증된 활용형에서 불필요하게 덧붙은 ‘ㄹ’을 바로잡습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `먹을려고` → `먹으려고`

<a id="grammar.ending.euryeo-context"></a>
## `grammar.ending.euryeo-context` — ‘갈려고/갈려면’ 문맥 검토

‘갈려고’, ‘갈려면’을 문맥에 따라 ‘가려고’, ‘가려면’으로 검토합니다.

- 분류: `grammar`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
- 예: `갈려고` → `가려고`

<a id="grammar.ending.hal-ge"></a>
## `grammar.ending.hal-ge` — 할게 표기

약속이나 의지를 나타내는 종결 어미 ‘-ㄹ게’를 바르게 씁니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할께` → `할게`

<a id="grammar.ending.seumnida"></a>
## `grammar.ending.seumnida` — 현대 표준 ‘-습니다/-ㅂ니다’

옛 표기 ‘-읍니다/-읍니까’를 받침에 맞는 현대 표준 활용으로 바로잡습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `읽읍니다` → `읽습니다`

<a id="grammar.ending.sipsio"></a>
## `grammar.ending.sipsio` — 높임 명령형 ‘-십시오’

높임 명령형의 잘못된 ‘-십시요’를 ‘-십시오’로 바로잡습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `확인하십시요` → `확인하십시오`

<a id="grammar.negation.an-before-predicate"></a>
## `grammar.negation.an-before-predicate` — 부정 부사 ‘안’

부정 부사 ‘안’을 사용하세요.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `않 간다` → `안 간다`

<a id="grammar.negation.anh-doe"></a>
## `grammar.negation.anh-doe` — 부정 부사 ‘안’과 ‘되다’

‘않되다/않돼다’처럼 잘못 쓴 부정을 ‘안 되다’ 계열로 바로잡습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `않됩니다` → `안 됩니다`

<a id="grammar.negation.ji-anh"></a>
## `grammar.negation.ji-anh` — ‘-지 않다’ 띄어쓰기

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
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
- 예: `책와` → `책과`

<a id="grammar.particle.duplicate"></a>
## `grammar.particle.duplicate` — 조사 중복

조사가 중복된 것 같습니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `자료를를 확인했다` → `자료를 확인했다`

<a id="grammar.particle.instrumental-allomorph"></a>
## `grammar.particle.instrumental-allomorph` — 부사격 조사 ‘으로/로’

앞말의 받침에 맞춰 부사격 조사 ‘으로/로’를 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
- 예: `책로` → `책으로`

<a id="grammar.particle.object-allomorph"></a>
## `grammar.particle.object-allomorph` — 목적격 조사 ‘을/를’

앞말의 받침에 맞춰 목적격 조사 ‘을/를’을 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
- 예: `책를` → `책을`

<a id="grammar.particle.subject-allomorph"></a>
## `grammar.particle.subject-allomorph` — 주격 조사 ‘이/가’

앞말의 받침에 맞춰 주격 조사 ‘이/가’를 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
- 예: `나무이` → `나무가`

<a id="grammar.particle.topic-allomorph"></a>
## `grammar.particle.topic-allomorph` — 보조사 ‘은/는’

앞말의 받침에 맞춰 보조사 ‘은/는’을 선택합니다.

- 분류: `grammar`
- 신뢰도: `high`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
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
## `punctuation.no-space-before-mark` — 문장 부호 앞 띄어쓰기

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
## `repetition.ending` — 종결 표현 반복

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

<a id="spacing.dependent-noun.beop"></a>
## `spacing.dependent-noun.beop` — 의존 명사 ‘법’ 띄어쓰기

관형형 뒤에서 일반적인 이치를 나타내는 ‘법’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `사는법이다` → `사는 법이다`

<a id="spacing.dependent-noun.chae"></a>
## `spacing.dependent-noun.chae` — 의존 명사 ‘채’ 띄어쓰기

관형형 뒤에서 상태를 나타내는 ‘채’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `입은채로` → `입은 채로`

<a id="spacing.dependent-noun.daero"></a>
## `spacing.dependent-noun.daero` — 의존 명사 ‘대로’ 띄어쓰기

관형형 뒤에서 양상·방식을 나타내는 ‘대로’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `들은대로` → `들은 대로`

<a id="spacing.dependent-noun.de"></a>
## `spacing.dependent-noun.de` — 의존 명사 ‘데’ 띄어쓰기

관형형 뒤에서 장소·경우를 나타내는 ‘데’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `묵을데가` → `묵을 데가`

<a id="spacing.dependent-noun.deut"></a>
## `spacing.dependent-noun.deut` — 의존 명사 ‘듯’ 띄어쓰기

관형형 뒤에서 짐작을 나타내는 ‘듯’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `모르는듯하다` → `모르는 듯하다`

<a id="spacing.dependent-noun.geot"></a>
## `spacing.dependent-noun.geot` — 의존 명사 ‘것’ 띄어쓰기

의존 명사 ‘것’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `좋을것 같다` → `좋을 것 같다`

<a id="spacing.dependent-noun.jeok"></a>
## `spacing.dependent-noun.jeok` — 의존 명사 ‘적’ 띄어쓰기

의존 명사 ‘적’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `본적 있다` → `본 적 있다`

<a id="spacing.dependent-noun.jul"></a>
## `spacing.dependent-noun.jul` — 의존 명사 ‘줄’ 띄어쓰기

의존 명사 ‘줄’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `알줄 안다` → `알 줄 안다`

<a id="spacing.dependent-noun.jung"></a>
## `spacing.dependent-noun.jung` — 의존 명사 ‘중’ 띄어쓰기

의존 명사 ‘중’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `하는중` → `하는 중`

<a id="spacing.dependent-noun.mankeum"></a>
## `spacing.dependent-noun.mankeum` — 의존 명사 ‘만큼’ 띄어쓰기

관형형 뒤에서 정도를 나타내는 ‘만큼’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `먹을만큼` → `먹을 만큼`

<a id="spacing.dependent-noun.ppun"></a>
## `spacing.dependent-noun.ppun` — 의존 명사 ‘뿐’ 띄어쓰기

의존 명사 ‘뿐’은 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `기다릴뿐이다` → `기다릴 뿐이다`

<a id="spacing.dependent-noun.ri"></a>
## `spacing.dependent-noun.ri` — 의존 명사 ‘리’ 띄어쓰기

관형형 뒤에서 가능성을 나타내는 ‘리’의 띄어쓰기를 검토합니다.

- 분류: `spacing`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `잊을리가 없다` → `잊을 리가 없다`

<a id="spacing.dependent-noun.su"></a>
## `spacing.dependent-noun.su` — 의존 명사 ‘수’ 띄어쓰기

의존 명사 ‘수’는 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할수 있다` → `할 수 있다`

<a id="spacing.dependent-noun.ttae"></a>
## `spacing.dependent-noun.ttae` — 의존 명사 ‘때’ 띄어쓰기

의존 명사 ‘때’는 앞말과 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `만날때` → `만날 때`

<a id="spacing.fixed.ppunman-anira"></a>
## `spacing.fixed.ppunman-anira` — ‘뿐만 아니라’ 띄어쓰기

‘뿐만 아니라’는 띄어 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `뿐만아니라` → `뿐만 아니라`

<a id="spacing.fixed.su-bakke"></a>
## `spacing.fixed.su-bakke` — ‘수밖에’ 붙여쓰기

‘수밖에’는 붙여 씁니다.

- 분류: `spacing`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `할 수 밖에` → `할 수밖에`

<a id="spelling.adverb.i-hi"></a>
## `spelling.adverb.i-hi` — 부사 ‘-이/-히’ 표기

부사의 ‘-이/-히’ 표기를 확인하세요.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `깨끗히` → `깨끗이`

<a id="spelling.confusable.oraen-oraet"></a>
## `spelling.confusable.oraen-oraet` — ‘오랜/오랫-’ 표기

‘오랫만에’는 ‘오랜만에’로 쓰는 것이 맞습니다.

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
## `spelling.confusable.wen-waen` — ‘웬/왠’ 구별

‘왠만’은 ‘웬만’으로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `왠만` → `웬만`

<a id="spelling.conjugation.boe-bwae"></a>
## `spelling.conjugation.boe-bwae` — ‘봬요’ 표기

‘뵈요’는 ‘봬요’로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `뵈요` → `봬요`

<a id="spelling.conjugation.dwaet"></a>
## `spelling.conjugation.dwaet` — ‘됐’ 표기

‘됬’은 ‘됐’으로 쓰는 것이 맞습니다.

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
## `spelling.lexical.anseong-matchum` — 안성맞춤 표기

‘안성마춤’은 ‘안성맞춤’으로 쓰는 것이 맞습니다.

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
## `spelling.lexical.chojeom` — 초점 표기

‘촛점’은 ‘초점’으로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `촛점` → `초점`

<a id="spelling.lexical.daega"></a>
## `spelling.lexical.daega` — 대가 표기

‘댓가’는 ‘대가’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.dodaeche` — 도대체 표기

‘도데체’는 ‘도대체’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.eoieopda` — 어이없다 표기

‘어의없다’는 ‘어이없다’로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `어의없` → `어이없`

<a id="spelling.lexical.eojjaetdeun"></a>
## `spelling.lexical.eojjaetdeun` — 어쨌든 표기

‘어쨋든’은 ‘어쨌든’으로 쓰는 것이 맞습니다.

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
## `spelling.lexical.gaesu` — 개수 표기

‘갯수’는 ‘개수’로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `갯수` → `개수`

<a id="spelling.lexical.geokkuro"></a>
## `spelling.lexical.geokkuro` — 거꾸로 표기

‘꺼꾸로’는 ‘거꾸로’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.geumse` — 금세 표기

‘금새’는 ‘금세’로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `금새` → `금세`

<a id="spelling.lexical.gop-ppaegi"></a>
## `spelling.lexical.gop-ppaegi` — 곱빼기 표기

‘곱배기’는 ‘곱빼기’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.hamatteomyeon` — 하마터면 표기

‘하마트면’은 ‘하마터면’으로 쓰는 것이 맞습니다.

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
## `spelling.lexical.huihanhada` — 희한하다 표기

‘희안’은 ‘희한’으로 쓰는 것이 맞습니다.

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
## `spelling.lexical.jjagipgi` — 짜깁기 표기

‘짜집기’는 ‘짜깁기’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.jjigae` — 찌개 표기

음식 이름 ‘찌개’의 표기를 확인하세요.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
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
## `spelling.lexical.tongjjaero` — 통째로 표기

‘통채로’는 ‘통째로’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.yeokhal` — 역할 표기

‘역활’은 ‘역할’로 쓰는 것이 맞습니다.

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
## `spelling.lexical.yukgaejang` — 육개장 표기

‘육계장’은 ‘육개장’으로 쓰는 것이 맞습니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `육계장` → `육개장`

<a id="spelling.loanword.curated"></a>
## `spelling.loanword.curated` — 표준 외래어 표기

표준 외래어 표기를 확인하세요.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `메세지` → `메시지`

<a id="style.redundancy.gajang-choego"></a>
## `style.redundancy.gajang-choego` — ‘가장 최고’ 의미 중복

‘가장 최고’는 뜻이 겹칠 수 있습니다. 문맥을 보고 한 표현만 남기세요.

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

<a id="spelling.lexical.deita"></a>
## `spelling.lexical.deita` — 데이터 표기

‘데이타’는 ‘데이터’로 씁니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `데이타` → `데이터`

<a id="spelling.lexical.seolreim"></a>
## `spelling.lexical.seolreim` — 설렘 표기

명사 ‘설렘’의 표기를 확인합니다.

- 분류: `spelling`
- 신뢰도: `high`
- 수정 안전도: `safe`
- 기본 활성화: `true`
- 프로필: `default`, `strict`, `editorial`
- 예: `설레임` → `설렘`

<a id="spelling.lexical.barem"></a>
## `spelling.lexical.barem` — 바람 표기 검토

문맥에 따라 ‘바램’을 ‘바람’으로 고치는지 검토합니다.

- 분류: `spelling`
- 신뢰도: `medium`
- 수정 안전도: `review`
- 기본 활성화: `false`
- 프로필: `strict`, `editorial`
- 예: `바램` → `바람`
