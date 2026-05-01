GST_TRACERS="pipeline-snapshot" GST_DEBUG_DUMP_DOT_DIR=. cargo run
kill -SIGUSR1 $(pidof gst-launch-1.0) to generate a .dot
dot -Tpng /tmp/0.00.*.dot -o graph.png
