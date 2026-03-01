# linked: REQ001, SPEC001, SPEC002, SPEC003
@REQ001
Feature: CLI タブ補完
  claude-registry CLI でスキル名・プロファイル名などをタブ補完できる。
  補完スクリプトは bash-completion の自動検出ディレクトリに配置され、
  .bashrc / .zshrc の変更は不要。

  Background:
    Given レジストリにスキル "bdd-behave-expert-skill" が存在する
    And レジストリにスキル "code-review" が存在する
    And レジストリにプロファイル "web-frontend" が存在する
    And レジストリにフック "desktop-notify" が存在する
    And レジストリにエージェント "repo-researcher" が存在する

  @SPEC001
  Scenario: _complete skills で利用可能なスキル名を出力する
    When "claude-registry _complete skills" を実行する
    Then 標準出力にスキル名が1行1件で出力される
    And 出力に "bdd-behave-expert-skill" が含まれる
    And 出力に "code-review" が含まれる

  @SPEC001
  Scenario: _complete profiles で利用可能なプロファイル名を出力する
    When "claude-registry _complete profiles" を実行する
    Then 標準出力にプロファイル名が1行1件で出力される
    And 出力に "web-frontend" が含まれる

  @SPEC001
  Scenario: _complete agents で利用可能なエージェント名を出力する
    When "claude-registry _complete agents" を実行する
    Then 標準出力にエージェント名が1行1件で出力される
    And 出力に "repo-researcher" が含まれる

  @SPEC001
  Scenario: _complete hooks で利用可能なフック名を出力する
    When "claude-registry _complete hooks" を実行する
    Then 標準出力にフック名が1行1件で出力される
    And 出力に "desktop-notify" が含まれる

  @SPEC001
  Scenario: install.sh が補完スクリプトを自動検出ディレクトリに配置する
    When install.sh を実行する
    Then "~/.local/share/bash-completion/completions/claude-registry" が作成される

  @SPEC002
  Scenario: skill install でスキル名が補完候補に表示される
    Given bash-completion が有効である
    When "claude-registry skill install " に対してタブ補完を実行する
    Then 補完候補に "bdd-behave-expert-skill" が含まれる
    And 補完候補に "code-review" が含まれる

  @SPEC002
  Scenario: agent install でエージェント名が補完候補に表示される
    Given bash-completion が有効である
    When "claude-registry agent install " に対してタブ補完を実行する
    Then 補完候補に "repo-researcher" が含まれる

  @SPEC003
  Scenario: profile install でプロファイル名が補完候補に表示される
    Given bash-completion が有効である
    When "claude-registry profile install " に対してタブ補完を実行する
    Then 補完候補に "web-frontend" が含まれる

  @SPEC003
  Scenario: hook install でフック名が補完候補に表示される
    Given bash-completion が有効である
    When "claude-registry hook install " に対してタブ補完を実行する
    Then 補完候補に "desktop-notify" が含まれる
