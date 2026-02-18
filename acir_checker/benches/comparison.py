#!/usr/bin/env python3

import json
import os
import argparse

# Load results from a directory and return a list of results sorted by filename
def load_results(results_dir):
    results = []
    for filename in os.listdir(results_dir):
        if filename.endswith('.json'):
            with open(os.path.join(results_dir, filename), 'r') as f:
                data = json.load(f)
                data['results'][0]["filename"] = filename
                results += [data['results'][0]]
                results.sort(key=lambda x: x['filename'])
    return results


def print_header_row():
    print("| Name | mean | min | max | successes | timeouts |")
    print("|------|------|-----|-----|-----------|----------|")

def print_row(tag,result):
    # exit_codes is a list of exit codes for each run,
    # where 0 means success and 124 means timeout
    print("| {}-{} | {} | {}| {} | {} | {} |"
          .format(tag, result['filename'], result['mean'], result['min'], result['max'], 
                  result['exit_codes'].count(0), result['exit_codes'].count(124)))

def print_row_placeholder(tag, filename):
    print("| {}-{} | * | * | * | * | * |".format(tag, filename))

# Compare results from two different runs and print a table with the results
# The results are printed in a table with the following columns:
# Name, mean, min, max, successes, timeouts
# If a row does not exist in one of the results, it is printed with a placeholder
# where values are replaced with "*"
def compare_results(tag1, results_dir1, tag2, results_dir2):
    results1 = {}
    results2 = {}

    results1 = load_results(results_dir1)
    results2 = load_results(results_dir2)

    i = 0
    j = 0
    print_header_row()
    while i < len(results1):
        result1 = results1[i]
        if j < len(results2):
            result2 = results2[j]
            if result1['filename'] < result2['filename']:
                print_row(tag1,result1)
                print_row_placeholder(tag2, result1['filename'])
            elif result1['filename'] > result2['filename']:
                print_row_placeholder(tag1, result2['filename'])
                print_row(tag2, result2)
                j += 1
                continue
            else:
                print_row(tag1, result1)
                print_row(tag2, result2)
                j += 1
        else:
            print_row(tag1, result1)
            print_row_placeholder(tag2, result1['filename'])
        i += 1

    return results1, results2

options="h"

if __name__ == "__main__":
    # Get parameters from input arguments
    parser = argparse.ArgumentParser(description="Compare benchmark results " \
        "from two different runs")
    parser.add_argument("--base", help="Base directory")
    parser.add_argument("--comp", help="Comparison directory")
    parser.add_argument("--tbase", help="Tag for base directory")
    parser.add_argument("--tcomp", help="Tag for comparison directory")
    args = parser.parse_args()
    compare_results(args.tbase, args.base, args.tcomp, args.comp)