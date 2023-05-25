import os
import json
import subprocess
from flask import Flask, render_template, request
import time

app = Flask(__name__)

# Set the contract address
contract = os.environ["CONSUMER_CONTRACT"]


def secretcli(args, json_output=True):
    cmd = ["secretcli"] + args
    if json_output:
        cmd.append("--output=json")
    result = subprocess.check_output(cmd)
    return json.loads(result) if json_output else result


def wait_for_query_change(prev_random):
    while True:
        response = {}
        try:
            response = secretcli(["q", "compute", "query", contract, '{"last_random": {}}'])
        except:
            pass

        if "random" in response and response["random"] != prev_random:
            return response

        time.sleep(1)


@app.route("/")
def index():
    return render_template("index.html")


@app.route("/refresh")
def get_current_values():
    try:
        response = secretcli(["q", "compute", "query", contract, '{"last_random": {}}'])["random"]
        random_value = response["random"]
        block_height = response["height"]
        return {"random_value": random_value, "block_height": block_height}
    except:
        # no value exists - we're waiting for 1st random
        response = wait_for_query_change(None)
        random_value = response["random"]
        block_height = response["height"]
        return {"random_value": random_value, "block_height": block_height}



@app.route("/update-random")
def update_random():
    prev_random = None
    try:
        prev_random = secretcli(["q", "compute", "query", contract, '{"last_random": {}}'])
    except:
        return {"error": "Still waiting for initial random"}

    try:
        tx = secretcli(["tx", "compute", "execute", contract, '{"do_something": {}}', "--from", "a", "--gas", "200000",
                        "-y"])
        response = wait_for_query_change(prev_random["random"])
        random_value = response["random"]
        block_height = response["height"]
        return {"random_value": random_value, "block_height": block_height}
    except Exception as e:
        return {"error": str(e)}


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8080)
