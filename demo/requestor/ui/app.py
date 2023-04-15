import os
import json
import subprocess
from flask import Flask, render_template
import time

app = Flask(__name__)

# Set the contract address
contract = "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg"
code_hash = "1e3d516013e80cfcdd5be8838833c2627698c9a678a3c2494917a2255277763a"


def secretcli(args, json_output=True):
    cmd = ["secretcli"] + args
    if json_output:
        cmd.append("--output=json")
    result = subprocess.check_output(cmd)
    return json.loads(result) if json_output else result


def wait_for_tx(tx_hash, message="Waiting for tx to finish on-chain..."):
    print(message)
    while True:
        try:
            result = secretcli(["q", "compute", "tx", tx_hash])
            return result
        except subprocess.CalledProcessError:
            print(".", end="")
            time.sleep(1)


@app.route("/")
def index():
    try:
        response = secretcli(["tx", "compute", "query", contract, '{"get_last_random": {}}'])
        random_value = response["result"]["random"]
        block_height = response["result"]["block_height"]
    except Exception as e:
        random_value = "Error: {}".format(e)
        block_height = "Error: {}".format(e)

    return render_template("index.html", random_value=random_value, block_height=block_height)


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8080)
