
use builtin;
use str;

set edit:completion:arg-completer[hime-tools] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'hime-tools'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'hime-tools'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand bin 'bin'
            cand nvsg 'nvsg'
            cand hcb 'hcb'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;bin'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand ls 'ls'
            cand get 'get'
            cand pack 'pack'
            cand unpack 'unpack'
            cand validate 'validate'
            cand replace 'replace'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;bin;ls'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;bin;get'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;bin;pack'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;bin;unpack'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;bin;validate'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;bin;replace'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;bin;help'= {
            cand ls 'ls'
            cand get 'get'
            cand pack 'pack'
            cand unpack 'unpack'
            cand validate 'validate'
            cand replace 'replace'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;bin;help;ls'= {
        }
        &'hime-tools;bin;help;get'= {
        }
        &'hime-tools;bin;help;pack'= {
        }
        &'hime-tools;bin;help;unpack'= {
        }
        &'hime-tools;bin;help;validate'= {
        }
        &'hime-tools;bin;help;replace'= {
        }
        &'hime-tools;bin;help;help'= {
        }
        &'hime-tools;nvsg'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand info 'info'
            cand decode 'decode'
            cand encode 'encode'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;nvsg;info'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;nvsg;decode'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;nvsg;encode'= {
            cand -x 'x'
            cand -y 'y'
            cand -u 'u'
            cand -v 'v'
            cand --type 'type'
            cand --parts 'parts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;nvsg;help'= {
            cand info 'info'
            cand decode 'decode'
            cand encode 'encode'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;nvsg;help;info'= {
        }
        &'hime-tools;nvsg;help;decode'= {
        }
        &'hime-tools;nvsg;help;encode'= {
        }
        &'hime-tools;nvsg;help;help'= {
        }
        &'hime-tools;hcb'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand disasm 'disasm'
            cand asm 'asm'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;hcb;disasm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;hcb;asm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'hime-tools;hcb;help'= {
            cand disasm 'disasm'
            cand asm 'asm'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;hcb;help;disasm'= {
        }
        &'hime-tools;hcb;help;asm'= {
        }
        &'hime-tools;hcb;help;help'= {
        }
        &'hime-tools;help'= {
            cand bin 'bin'
            cand nvsg 'nvsg'
            cand hcb 'hcb'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'hime-tools;help;bin'= {
            cand ls 'ls'
            cand get 'get'
            cand pack 'pack'
            cand unpack 'unpack'
            cand validate 'validate'
            cand replace 'replace'
        }
        &'hime-tools;help;bin;ls'= {
        }
        &'hime-tools;help;bin;get'= {
        }
        &'hime-tools;help;bin;pack'= {
        }
        &'hime-tools;help;bin;unpack'= {
        }
        &'hime-tools;help;bin;validate'= {
        }
        &'hime-tools;help;bin;replace'= {
        }
        &'hime-tools;help;nvsg'= {
            cand info 'info'
            cand decode 'decode'
            cand encode 'encode'
        }
        &'hime-tools;help;nvsg;info'= {
        }
        &'hime-tools;help;nvsg;decode'= {
        }
        &'hime-tools;help;nvsg;encode'= {
        }
        &'hime-tools;help;hcb'= {
            cand disasm 'disasm'
            cand asm 'asm'
        }
        &'hime-tools;help;hcb;disasm'= {
        }
        &'hime-tools;help;hcb;asm'= {
        }
        &'hime-tools;help;help'= {
        }
    ]
    $completions[$command]
}
