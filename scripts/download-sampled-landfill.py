#!/usr/bin/env python3

import boto3
import base64
import logging
import json
import os


def parse_schema_name(path):
    """Given a directory path to a json schema in the mps directory, generate
    the fully qualified name in the form `{namespace}.{doctype}.{docver}`."""
    elements = path.split("/")
    doctype, docver = elements[-1].split(".")[:-2]
    namespace = elements[-3]
    return f"{namespace}.{doctype}.{docver}"


def construct_schema_set(path):
    """return a set containing "{namespace}.{doctype}.{doctype}" strings"""
    schemas = set()
    for root, _, files in os.walk(path):
        for name in files:
            if name.endswith(".schema.json"):
                schemas.add(parse_schema_name(os.path.join(root, name)))
    return schemas


def get_schema_name(key):
    # Example:
    # sanitized-landfill-sample/v3/submission_date_s3=20190308/namespace=webpagetest/doc_type=webpagetest-run/doc_version=1/part-00122-tid-2954272513278013416-c06a39af-9979-41a5-8459-76412a4554b3-650.c000.json
    params = dict([x.split("=") for x in key.split("/") if "=" in x])
    return ".".join(map(params.get, ["namespace", "doc_type", "doc_version"]))


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)

    root = os.path.join(os.path.dirname(__file__), "..")
    os.chdir(root)

    # current directory
    schemas = construct_schema_set("schemas")

    output_folder = "data"
    if not os.path.exists(output_folder):
        os.mkdir(output_folder)

    bucket = "telemetry-parquet"
    prefix = "sanitized-landfill-sample/v3/submission_date_s3=20190310"
    s3 = boto3.client("s3")

    objs = s3.list_objects(Bucket=bucket, Prefix=prefix)
    keys = [obj["Key"] for obj in objs["Contents"] if obj["Key"].endswith(".json")]

    for key in keys:
        schema_name = get_schema_name(key)
        if not schema_name in schemas:
            logging.info("schema does not exist for {}".format(schema_name))
            continue

        logging.info("Creating ndjson for {}".format(schema_name))
        data = (
            s3.get_object(Bucket=bucket, Key=key)["Body"].read().decode("utf-8").strip()
        )
        lines = data.split("\n")

        with open(f"{output_folder}/{schema_name}.ndjson", "w") as fp:
            for line in lines:
                # each of the lines contains metadata with a content field
                content = json.loads(line).get("content")
                if not content:
                    continue
                fp.write(json.dumps(json.loads(content)) + "\n")
        logging.info("Wrote {} documents".format(len(lines)))
    logging.info("Done!")
