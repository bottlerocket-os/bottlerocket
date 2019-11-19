SHELL := bash
stacks := $(wildcard *.yml)

list:
	@printf "%s\n" $(stacks)

validate: $(addprefix validate/,$(stacks))

validate/%:
	$(info validating stack: $*)
	aws cloudformation validate-template --template-body "$$(< $*)"

.PHONY: test list
