Write a Python script in ./ops/evaluate.py that
- implements the evaluation described in kpop_prompt.md
- command 'all': prints all metrics for all doc types for all parsers on stdout
- command 'rpdf': same but just for rpdf
- uses click for commands
- uses multiprocessing to speed up evaluation
- reports time taken to run evals
- runs via the command "./ops/evaluate.py" (executable, shebang)