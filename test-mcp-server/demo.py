#!/usr/bin/env python3
"""
Quick demo script that shows the test server and client working together
"""

import subprocess
import time
import sys
import os

def run_demo():
    print("=== MCP Test Server Demo ===\n")
    
    # Start server
    print("1. Starting test server...")
    server_process = subprocess.Popen(
        [sys.executable, "test_server.py", "--fast"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    time.sleep(0.5)
    
    # Start client and pipe to server
    print("2. Starting test client...")
    client_process = subprocess.Popen(
        [sys.executable, "test_client.py"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    print("3. Running communication test...\n")
    
    try:
        # Get client output and send to server
        client_output, client_stderr = client_process.communicate(timeout=10)
        
        if client_output:
            # Send client requests to server
            server_output, server_stderr = server_process.communicate(
                input=client_output, timeout=10
            )
            
            print("✅ Client sent requests:")
            for line in client_stderr.split('\n'):
                if line.strip():
                    print(f"   {line}")
            
            print("\n✅ Server processed requests:")
            for line in server_stderr.split('\n'):
                if line.strip():
                    print(f"   {line}")
            
            print("\n✅ Sample JSON-RPC responses:")
            responses = server_output.strip().split('\n')
            for i, response in enumerate(responses[:3]):  # Show first 3 responses
                if response.strip():
                    print(f"   Response {i+1}: {response[:100]}...")
            
            print(f"\n✅ Demo completed! Server handled {len(responses)} requests.")
            print("\nTo see this in the MCP Monitor:")
            print("1. Run: ./run_tests.sh monitor")
            print("2. In another terminal: ./run_tests.sh proxy --fast") 
            print("3. In a third terminal: ./run_tests.sh client")
        
    except subprocess.TimeoutExpired:
        print("❌ Demo timed out")
        client_process.kill()
        server_process.kill()
    except Exception as e:
        print(f"❌ Demo error: {e}")
    finally:
        # Cleanup
        if client_process.poll() is None:
            client_process.terminate()
        if server_process.poll() is None:
            server_process.terminate()

if __name__ == "__main__":
    run_demo()