LATEXMK ?= latexmk

.PHONY: proposal.pdf
proposal.pdf:
	cd proposal && $(LATEXMK) main.tex
	cp proposal/build/main.pdf proposal.pdf

.PHONY: slides.pdf
slides.pdf:
	cd slides && $(LATEXMK) main.tex
	cp slides/build/main.pdf slides.pdf
