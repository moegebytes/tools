# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_hime_tools_global_optspecs
	string join \n h/help
end

function __fish_hime_tools_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_hime_tools_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_hime_tools_using_subcommand
	set -l cmd (__fish_hime_tools_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c hime-tools -n "__fish_hime_tools_needs_command" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_needs_command" -f -a "bin"
complete -c hime-tools -n "__fish_hime_tools_needs_command" -f -a "nvsg"
complete -c hime-tools -n "__fish_hime_tools_needs_command" -f -a "hcb"
complete -c hime-tools -n "__fish_hime_tools_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "ls"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "get"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "pack"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "unpack"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "validate"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "replace"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and not __fish_seen_subcommand_from ls get pack unpack validate replace help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from get" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from pack" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from unpack" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from validate" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from replace" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "ls"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "get"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "pack"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "unpack"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "validate"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "replace"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand bin; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and not __fish_seen_subcommand_from info decode encode help" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and not __fish_seen_subcommand_from info decode encode help" -f -a "info"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and not __fish_seen_subcommand_from info decode encode help" -f -a "decode"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and not __fish_seen_subcommand_from info decode encode help" -f -a "encode"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and not __fish_seen_subcommand_from info decode encode help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from info" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from decode" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -s x -r
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -s y -r
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -s u -r
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -s v -r
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -l type -r
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -l parts -r
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from encode" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from help" -f -a "info"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from help" -f -a "decode"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from help" -f -a "encode"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand nvsg; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and not __fish_seen_subcommand_from disasm asm help" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and not __fish_seen_subcommand_from disasm asm help" -f -a "disasm"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and not __fish_seen_subcommand_from disasm asm help" -f -a "asm"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and not __fish_seen_subcommand_from disasm asm help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and __fish_seen_subcommand_from disasm" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and __fish_seen_subcommand_from asm" -s h -l help -d 'Print help'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and __fish_seen_subcommand_from help" -f -a "disasm"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and __fish_seen_subcommand_from help" -f -a "asm"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand hcb; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and not __fish_seen_subcommand_from bin nvsg hcb help" -f -a "bin"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and not __fish_seen_subcommand_from bin nvsg hcb help" -f -a "nvsg"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and not __fish_seen_subcommand_from bin nvsg hcb help" -f -a "hcb"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and not __fish_seen_subcommand_from bin nvsg hcb help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from bin" -f -a "ls"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from bin" -f -a "get"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from bin" -f -a "pack"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from bin" -f -a "unpack"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from bin" -f -a "validate"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from bin" -f -a "replace"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from nvsg" -f -a "info"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from nvsg" -f -a "decode"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from nvsg" -f -a "encode"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from hcb" -f -a "disasm"
complete -c hime-tools -n "__fish_hime_tools_using_subcommand help; and __fish_seen_subcommand_from hcb" -f -a "asm"
