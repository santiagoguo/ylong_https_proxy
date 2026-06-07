#!/bin/bash
# Simple libcurl performance comparison
# Measures the time to perform 100 concurrent proxy connections

PROXY="127.0.0.1:19999"
NUM_REQUESTS=100

echo "🚀 Starting libcurl comparison..."
echo "   Target: Mock Proxy on $PROXY"
echo "   Requests: $NUM_REQUESTS"

# Since our mock proxy just returns 200 OK and closes,
# we measure the time it takes for curl to connect and get the response.
# We use curl's -w flag to output time_total

total_time=0
for i in $(seq 1 $NUM_REQUESTS); do
    # Run curl in background
    time=$(curl -s -o /dev/null -w "%{time_total}" -x http://$PROXY http://example.com 2>/dev/null)
    total_time=$(echo "$total_time + $time" | bc)
done

avg=$(echo "scale=4; $total_time / $NUM_REQUESTS" | bc)
echo "   libcurl Avg Time: ${avg}s"
echo "✅ libcurl benchmark complete."
