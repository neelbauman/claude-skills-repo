# bash completion for claude-registry
# Install: copy to ~/.local/share/bash-completion/completions/claude-registry

_claude_registry() {
    local cur prev words cword
    _init_completion || return

    local top_commands="skill agent catalog profile hook"

    # Determine position context
    local cmd="" subcmd=""
    local i
    for ((i = 1; i < cword; i++)); do
        case "${words[i]}" in
            skill|agent|catalog|profile|hook)
                cmd="${words[i]}"
                ;;
            install|list|new|available|uninstall|build)
                subcmd="${words[i]}"
                ;;
        esac
    done

    # Top-level completion
    if [[ -z "$cmd" ]]; then
        COMPREPLY=($(compgen -W "$top_commands" -- "$cur"))
        return
    fi

    # Sub-action completion
    if [[ -z "$subcmd" ]]; then
        case "$cmd" in
            skill)
                COMPREPLY=($(compgen -W "install list new available uninstall" -- "$cur"))
                ;;
            agent)
                COMPREPLY=($(compgen -W "install list available uninstall" -- "$cur"))
                ;;
            catalog)
                COMPREPLY=($(compgen -W "build" -- "$cur"))
                ;;
            profile)
                COMPREPLY=($(compgen -W "install list" -- "$cur"))
                ;;
            hook)
                COMPREPLY=($(compgen -W "install uninstall list available new" -- "$cur"))
                ;;
        esac
        return
    fi

    # Option completion
    if [[ "$cur" == -* ]]; then
        case "$cmd" in
            skill)
                case "$subcmd" in
                    install)  COMPREPLY=($(compgen -W "--target --dry-run --help" -- "$cur")) ;;
                    list)     COMPREPLY=($(compgen -W "--target --help" -- "$cur")) ;;
                    new)      COMPREPLY=($(compgen -W "--description --help" -- "$cur")) ;;
                    uninstall) COMPREPLY=($(compgen -W "--target --help" -- "$cur")) ;;
                esac
                ;;
            agent)
                case "$subcmd" in
                    install)  COMPREPLY=($(compgen -W "--target --dry-run --help" -- "$cur")) ;;
                    list)     COMPREPLY=($(compgen -W "--target --help" -- "$cur")) ;;
                    uninstall) COMPREPLY=($(compgen -W "--target --help" -- "$cur")) ;;
                esac
                ;;
            profile)
                case "$subcmd" in
                    install)  COMPREPLY=($(compgen -W "--target --dry-run --help" -- "$cur")) ;;
                esac
                ;;
            hook)
                case "$subcmd" in
                    install)  COMPREPLY=($(compgen -W "--global --target --dry-run --help" -- "$cur")) ;;
                    uninstall) COMPREPLY=($(compgen -W "--global --target --help" -- "$cur")) ;;
                    list)     COMPREPLY=($(compgen -W "--global --target --help" -- "$cur")) ;;
                    new)      COMPREPLY=($(compgen -W "--description --help" -- "$cur")) ;;
                esac
                ;;
        esac
        return
    fi

    # Dynamic value completion (skill names, profile names, etc.)
    case "$cmd" in
        skill)
            case "$subcmd" in
                install|uninstall)
                    local candidates
                    candidates=$(claude-registry _complete skills 2>/dev/null)
                    COMPREPLY=($(compgen -W "$candidates" -- "$cur"))
                    ;;
            esac
            ;;
        agent)
            case "$subcmd" in
                install|uninstall)
                    local candidates
                    candidates=$(claude-registry _complete agents 2>/dev/null)
                    COMPREPLY=($(compgen -W "$candidates" -- "$cur"))
                    ;;
            esac
            ;;
        profile)
            case "$subcmd" in
                install)
                    local candidates
                    candidates=$(claude-registry _complete profiles 2>/dev/null)
                    COMPREPLY=($(compgen -W "$candidates" -- "$cur"))
                    ;;
            esac
            ;;
        hook)
            case "$subcmd" in
                install|uninstall)
                    local candidates
                    candidates=$(claude-registry _complete hooks 2>/dev/null)
                    COMPREPLY=($(compgen -W "$candidates" -- "$cur"))
                    ;;
            esac
            ;;
    esac
}

complete -F _claude_registry claude-registry
