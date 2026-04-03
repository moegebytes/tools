
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'hime-tools' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'hime-tools'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'hime-tools' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('bin', 'bin', [CompletionResultType]::ParameterValue, 'bin')
            [CompletionResult]::new('nvsg', 'nvsg', [CompletionResultType]::ParameterValue, 'nvsg')
            [CompletionResult]::new('hcb', 'hcb', [CompletionResultType]::ParameterValue, 'hcb')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;bin' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'ls')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'get')
            [CompletionResult]::new('pack', 'pack', [CompletionResultType]::ParameterValue, 'pack')
            [CompletionResult]::new('unpack', 'unpack', [CompletionResultType]::ParameterValue, 'unpack')
            [CompletionResult]::new('validate', 'validate', [CompletionResultType]::ParameterValue, 'validate')
            [CompletionResult]::new('replace', 'replace', [CompletionResultType]::ParameterValue, 'replace')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;bin;ls' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;bin;get' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;bin;pack' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;bin;unpack' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;bin;validate' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;bin;replace' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;bin;help' {
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'ls')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'get')
            [CompletionResult]::new('pack', 'pack', [CompletionResultType]::ParameterValue, 'pack')
            [CompletionResult]::new('unpack', 'unpack', [CompletionResultType]::ParameterValue, 'unpack')
            [CompletionResult]::new('validate', 'validate', [CompletionResultType]::ParameterValue, 'validate')
            [CompletionResult]::new('replace', 'replace', [CompletionResultType]::ParameterValue, 'replace')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;bin;help;ls' {
            break
        }
        'hime-tools;bin;help;get' {
            break
        }
        'hime-tools;bin;help;pack' {
            break
        }
        'hime-tools;bin;help;unpack' {
            break
        }
        'hime-tools;bin;help;validate' {
            break
        }
        'hime-tools;bin;help;replace' {
            break
        }
        'hime-tools;bin;help;help' {
            break
        }
        'hime-tools;nvsg' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'info')
            [CompletionResult]::new('decode', 'decode', [CompletionResultType]::ParameterValue, 'decode')
            [CompletionResult]::new('encode', 'encode', [CompletionResultType]::ParameterValue, 'encode')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;nvsg;info' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;nvsg;decode' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;nvsg;encode' {
            [CompletionResult]::new('-x', '-x', [CompletionResultType]::ParameterName, 'x')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'y')
            [CompletionResult]::new('-u', '-u', [CompletionResultType]::ParameterName, 'u')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'v')
            [CompletionResult]::new('--type', '--type', [CompletionResultType]::ParameterName, 'type')
            [CompletionResult]::new('--parts', '--parts', [CompletionResultType]::ParameterName, 'parts')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;nvsg;help' {
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'info')
            [CompletionResult]::new('decode', 'decode', [CompletionResultType]::ParameterValue, 'decode')
            [CompletionResult]::new('encode', 'encode', [CompletionResultType]::ParameterValue, 'encode')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;nvsg;help;info' {
            break
        }
        'hime-tools;nvsg;help;decode' {
            break
        }
        'hime-tools;nvsg;help;encode' {
            break
        }
        'hime-tools;nvsg;help;help' {
            break
        }
        'hime-tools;hcb' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('disasm', 'disasm', [CompletionResultType]::ParameterValue, 'disasm')
            [CompletionResult]::new('asm', 'asm', [CompletionResultType]::ParameterValue, 'asm')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;hcb;disasm' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;hcb;asm' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'hime-tools;hcb;help' {
            [CompletionResult]::new('disasm', 'disasm', [CompletionResultType]::ParameterValue, 'disasm')
            [CompletionResult]::new('asm', 'asm', [CompletionResultType]::ParameterValue, 'asm')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;hcb;help;disasm' {
            break
        }
        'hime-tools;hcb;help;asm' {
            break
        }
        'hime-tools;hcb;help;help' {
            break
        }
        'hime-tools;help' {
            [CompletionResult]::new('bin', 'bin', [CompletionResultType]::ParameterValue, 'bin')
            [CompletionResult]::new('nvsg', 'nvsg', [CompletionResultType]::ParameterValue, 'nvsg')
            [CompletionResult]::new('hcb', 'hcb', [CompletionResultType]::ParameterValue, 'hcb')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'hime-tools;help;bin' {
            [CompletionResult]::new('ls', 'ls', [CompletionResultType]::ParameterValue, 'ls')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'get')
            [CompletionResult]::new('pack', 'pack', [CompletionResultType]::ParameterValue, 'pack')
            [CompletionResult]::new('unpack', 'unpack', [CompletionResultType]::ParameterValue, 'unpack')
            [CompletionResult]::new('validate', 'validate', [CompletionResultType]::ParameterValue, 'validate')
            [CompletionResult]::new('replace', 'replace', [CompletionResultType]::ParameterValue, 'replace')
            break
        }
        'hime-tools;help;bin;ls' {
            break
        }
        'hime-tools;help;bin;get' {
            break
        }
        'hime-tools;help;bin;pack' {
            break
        }
        'hime-tools;help;bin;unpack' {
            break
        }
        'hime-tools;help;bin;validate' {
            break
        }
        'hime-tools;help;bin;replace' {
            break
        }
        'hime-tools;help;nvsg' {
            [CompletionResult]::new('info', 'info', [CompletionResultType]::ParameterValue, 'info')
            [CompletionResult]::new('decode', 'decode', [CompletionResultType]::ParameterValue, 'decode')
            [CompletionResult]::new('encode', 'encode', [CompletionResultType]::ParameterValue, 'encode')
            break
        }
        'hime-tools;help;nvsg;info' {
            break
        }
        'hime-tools;help;nvsg;decode' {
            break
        }
        'hime-tools;help;nvsg;encode' {
            break
        }
        'hime-tools;help;hcb' {
            [CompletionResult]::new('disasm', 'disasm', [CompletionResultType]::ParameterValue, 'disasm')
            [CompletionResult]::new('asm', 'asm', [CompletionResultType]::ParameterValue, 'asm')
            break
        }
        'hime-tools;help;hcb;disasm' {
            break
        }
        'hime-tools;help;hcb;asm' {
            break
        }
        'hime-tools;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
