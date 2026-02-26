use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Application input modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    FilterList,
    Command,
    SearchInput,
}

/// Messages representing user actions.
#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    // Navigation
    ScrollDown,
    ScrollUp,
    ScrollRight,
    ScrollLeft,
    GoToBottom,
    GoToTop,

    // Command mode
    EnterCommand,
    CancelCommand,
    SubmitCommand,
    CommandTypeChar(char),
    CommandBackspace,
    CommandComplete,

    // Search
    EnterSearch,
    CancelSearch,
    SubmitSearch,
    SearchTypeChar(char),
    SearchBackspace,
    NextMatch,
    PrevMatch,
    ClearSearch,

    // Selection
    ToggleSelection,
    YankSelection,
    ClearSelection,

    // Filter list
    FilterListDown,
    FilterListUp,
    DeleteSelectedFilter,
    CloseFilterList,

    // View options
    ToggleWrap,

    // Application
    Quit,
    NoOp,
}

/// Translate a key event into a message based on current mode.
pub fn translate(key: KeyEvent, mode: Mode) -> Option<Msg> {
    match mode {
        Mode::Normal => translate_normal(key),
        Mode::Command => translate_command(key),
        Mode::FilterList => translate_filter_list(key),
        Mode::SearchInput => translate_search(key),
    }
}

fn translate_normal(key: KeyEvent) -> Option<Msg> {
    // Check for Ctrl+C first
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Msg::Quit);
    }

    // Only process keys without modifiers (except for special cases)
    if !key.modifiers.is_empty() && !key.modifiers.contains(KeyModifiers::SHIFT) {
        return None;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Msg::ScrollDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Msg::ScrollUp),
        KeyCode::Char('l') | KeyCode::Right => Some(Msg::ScrollRight),
        KeyCode::Char('h') | KeyCode::Left => Some(Msg::ScrollLeft),
        KeyCode::Char('G') => Some(Msg::GoToBottom),
        KeyCode::Char('g') => Some(Msg::GoToTop),
        KeyCode::Char(':') => Some(Msg::EnterCommand),
        KeyCode::Char('w') => Some(Msg::ToggleWrap),
        KeyCode::Char('x') => Some(Msg::ToggleSelection),
        KeyCode::Char('y') => Some(Msg::YankSelection),
        KeyCode::Esc => Some(Msg::ClearSelection),
        KeyCode::Char('/') => Some(Msg::EnterSearch),
        KeyCode::Char('n') => Some(Msg::NextMatch),
        KeyCode::Char('N') => Some(Msg::PrevMatch),
        _ => None,
    }
}

fn translate_command(key: KeyEvent) -> Option<Msg> {
    // Handle Ctrl+C for quit (consistent with Normal mode)
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Msg::Quit);
    }

    match key.code {
        KeyCode::Esc => Some(Msg::CancelCommand),
        KeyCode::Enter => Some(Msg::SubmitCommand),
        KeyCode::Backspace => Some(Msg::CommandBackspace),
        KeyCode::Tab => Some(Msg::CommandComplete),
        KeyCode::Char(c) => Some(Msg::CommandTypeChar(c)),
        _ => None,
    }
}

fn translate_filter_list(key: KeyEvent) -> Option<Msg> {
    // Handle Ctrl+C for quit (consistent with Normal mode)
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Msg::Quit);
    }

    // Only process keys without modifiers (except Shift)
    if !key.modifiers.is_empty() && !key.modifiers.contains(KeyModifiers::SHIFT) {
        return None;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Msg::FilterListDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Msg::FilterListUp),
        KeyCode::Char('d') => Some(Msg::DeleteSelectedFilter),
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => Some(Msg::CloseFilterList),
        _ => None,
    }
}

fn translate_search(key: KeyEvent) -> Option<Msg> {
    // Handle Ctrl+C for quit (consistent with Normal mode)
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Msg::Quit);
    }

    match key.code {
        KeyCode::Esc => Some(Msg::CancelSearch),
        KeyCode::Enter => Some(Msg::SubmitSearch),
        KeyCode::Backspace => Some(Msg::SearchBackspace),
        KeyCode::Char(c) => Some(Msg::SearchTypeChar(c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_char(c: char) -> KeyEvent {
        KeyEvent::from(KeyCode::Char(c))
    }

    fn key_code(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn ctrl_c() -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_normal_mode_navigation() {
        assert_eq!(
            translate(key_char('j'), Mode::Normal),
            Some(Msg::ScrollDown)
        );
        assert_eq!(translate(key_char('k'), Mode::Normal), Some(Msg::ScrollUp));
        assert_eq!(
            translate(key_char('h'), Mode::Normal),
            Some(Msg::ScrollLeft)
        );
        assert_eq!(
            translate(key_char('l'), Mode::Normal),
            Some(Msg::ScrollRight)
        );
    }

    #[test]
    fn test_normal_mode_movement() {
        assert_eq!(
            translate(key_char('G'), Mode::Normal),
            Some(Msg::GoToBottom)
        );
        assert_eq!(translate(key_char('g'), Mode::Normal), Some(Msg::GoToTop));
    }

    #[test]
    fn test_normal_mode_mode_changes() {
        assert_eq!(
            translate(key_char(':'), Mode::Normal),
            Some(Msg::EnterCommand)
        );
        assert_eq!(
            translate(key_char('/'), Mode::Normal),
            Some(Msg::EnterSearch)
        );
    }

    #[test]
    fn test_normal_mode_quit() {
        assert_eq!(translate(ctrl_c(), Mode::Normal), Some(Msg::Quit));
    }

    #[test]
    fn test_command_mode() {
        assert_eq!(
            translate(key_code(KeyCode::Esc), Mode::Command),
            Some(Msg::CancelCommand)
        );
        assert_eq!(
            translate(key_code(KeyCode::Enter), Mode::Command),
            Some(Msg::SubmitCommand)
        );
        assert_eq!(
            translate(key_code(KeyCode::Backspace), Mode::Command),
            Some(Msg::CommandBackspace)
        );
        assert_eq!(
            translate(key_code(KeyCode::Tab), Mode::Command),
            Some(Msg::CommandComplete)
        );
        assert_eq!(
            translate(key_char('a'), Mode::Command),
            Some(Msg::CommandTypeChar('a'))
        );
    }

    #[test]
    fn test_search_mode() {
        assert_eq!(
            translate(key_code(KeyCode::Esc), Mode::SearchInput),
            Some(Msg::CancelSearch)
        );
        assert_eq!(
            translate(key_code(KeyCode::Enter), Mode::SearchInput),
            Some(Msg::SubmitSearch)
        );
        assert_eq!(
            translate(key_code(KeyCode::Backspace), Mode::SearchInput),
            Some(Msg::SearchBackspace)
        );
        assert_eq!(
            translate(key_char('x'), Mode::SearchInput),
            Some(Msg::SearchTypeChar('x'))
        );
    }

    #[test]
    fn test_filter_list_mode() {
        assert_eq!(
            translate(key_char('j'), Mode::FilterList),
            Some(Msg::FilterListDown)
        );
        assert_eq!(
            translate(key_char('k'), Mode::FilterList),
            Some(Msg::FilterListUp)
        );
        assert_eq!(
            translate(key_char('d'), Mode::FilterList),
            Some(Msg::DeleteSelectedFilter)
        );
        assert_eq!(
            translate(key_char('q'), Mode::FilterList),
            Some(Msg::CloseFilterList)
        );
        assert_eq!(
            translate(key_code(KeyCode::Esc), Mode::FilterList),
            Some(Msg::CloseFilterList)
        );
    }

    #[test]
    fn test_normal_mode_selection() {
        assert_eq!(
            translate(key_char('x'), Mode::Normal),
            Some(Msg::ToggleSelection)
        );
        assert_eq!(
            translate(key_char('y'), Mode::Normal),
            Some(Msg::YankSelection)
        );
        assert_eq!(
            translate(key_code(KeyCode::Esc), Mode::Normal),
            Some(Msg::ClearSelection)
        );
    }

    #[test]
    fn test_normal_mode_search_navigation() {
        assert_eq!(translate(key_char('n'), Mode::Normal), Some(Msg::NextMatch));
        assert_eq!(translate(key_char('N'), Mode::Normal), Some(Msg::PrevMatch));
    }

    #[test]
    fn test_normal_mode_view() {
        assert_eq!(
            translate(key_char('w'), Mode::Normal),
            Some(Msg::ToggleWrap)
        );
    }

    #[test]
    fn test_unknown_keys_return_none() {
        assert_eq!(translate(key_char('z'), Mode::Normal), None);
        assert_eq!(translate(key_char('1'), Mode::Normal), None);
    }
}
