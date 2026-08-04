use geullint_core::{StyleContext, StyleProfile};

#[test]
fn detects_code_and_formal_context_without_guessing_a_user_identity() {
    assert_eq!(
        StyleContext::detect("fn main() { println!(\"안녕\"); }").profile,
        StyleProfile::Code
    );
    assert_eq!(
        StyleContext::detect("본 문서는 다음과 같이 규정한다.").profile,
        StyleProfile::Formal
    );
    assert_eq!(
        StyleContext::detect("오늘은 날씨가 좋다.").profile,
        StyleProfile::Plain
    );
}

#[test]
fn style_context_reports_bounded_document_statistics() {
    let context = StyleContext::detect("첫 문장입니다. 둘째 문장입니다.");
    assert_eq!(context.sentence_count, 2);
    assert!(context.average_sentence_length > 0.0);
    assert!(context.average_sentence_length < 100.0);
}
