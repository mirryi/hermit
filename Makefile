LATEXMK ?= latexmk

.PHONY: proposal.pdf
proposal.pdf:
	cd proposal && $(LATEXMK) main.tex
	cp proposal/build/main.pdf proposal.pdf
