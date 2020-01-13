SHELL := bash
stacks := $(wildcard *.yml */*.yml)

list:
	@printf "%s\n" $(stacks)

validate: $(addprefix validate/,$(stacks))

validate/%:
	$(info validating stack: $*)
	aws cloudformation validate-template --template-body "$$(< $*)"

check: check-readme

check-readme:
	@$(foreach stack,$(stacks:.yml=),\
		grep -Fw -q -e "$(stack)" README.md ||\
		{ MISSING=1; echo "Missing README section mentioning $(stack)";  }; )\
	$${MISSING:+exit 1}

.PHONY: test list
