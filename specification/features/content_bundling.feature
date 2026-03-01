# linked: REQ002, SPEC004, SPEC005, SPEC006
@REQ002
Feature: コンテンツの同梱配布
  claude-registry のインストール時にスキル・エージェント・フック・プロファイルも
  同梱配布され、どのディレクトリからでも利用できる。

  Background:
    Given claude-registry が ~/.local/bin/claude-registry にインストールされている

  @SPEC004
  Scenario: install.sh 実行後にコンテンツが所定のパスに配置される
    When install.sh を実行する
    Then "~/.local/share/claude-registry/claude/skills/" が作成される
    And "~/.local/share/claude-registry/claude/agents/" が作成される
    And "~/.local/share/claude-registry/claude/hooks/" が作成される
    And "~/.local/share/claude-registry/profiles/" が作成される

  @SPEC004
  Scenario: コンテンツインストールは既存ファイルを上書きする
    Given "~/.local/share/claude-registry/claude/skills/code-review" が既に存在する
    When install.sh を再実行する
    Then "~/.local/share/claude-registry/claude/skills/code-review" が更新される

  @SPEC004
  Scenario: ユーザーが手動追加したコンテンツは削除されない
    Given "~/.local/share/claude-registry/claude/skills/my-custom-skill" が存在する
    When install.sh を再実行する
    Then "~/.local/share/claude-skills-repo/claude/skills/my-custom-skill" が残存する

  @SPEC005
  Scenario: CLAUDE_REGISTRY_ROOT 未設定時はインストール済みコンテンツが使われる
    Given CLAUDE_REGISTRY_ROOT 環境変数が設定されていない
    And "~/.local/share/claude-registry/" が存在する
    When "claude-registry skill available" を実行する
    Then "~/.local/share/claude-registry/claude/skills/" のスキルが表示される

  @SPEC005
  Scenario: CLAUDE_REGISTRY_ROOT が設定されている場合はそちらを優先する
    Given CLAUDE_REGISTRY_ROOT 環境変数が "/custom/registry" に設定されている
    When "claude-registry skill available" を実行する
    Then "/custom/registry/claude/skills/" のスキルが表示される

  @SPEC005
  Scenario: インストール済みコンテンツが存在しない場合はカレントディレクトリを使う
    Given CLAUDE_REGISTRY_ROOT 環境変数が設定されていない
    And "~/.local/share/claude-registry/" が存在しない
    When "claude-registry skill available" を実行する
    Then カレントディレクトリの "claude/skills/" のスキルが表示される

  @SPEC006
  Scenario: リリースアーカイブにコンテンツディレクトリが含まれる
    Given GitHub Releases から最新の tar.gz をダウンロードする
    When アーカイブを展開する
    Then "claude/" ディレクトリが含まれる
    And "profiles/" ディレクトリが含まれる
    And "completions/" ディレクトリが含まれる
    And "claude-registry" バイナリが含まれる
