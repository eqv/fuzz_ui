import sys
import time
import random
import requests
import glob
import base64

class IjonWebAPI:
    def __init__(self, fuzz_type, job_name, base_url):
        self.base_url = base_url.strip("/")
        self.fuzz_type = fuzz_type
        self.job_name = job_name

    def register(self):
        
        self.client_info = {
                "id": None,
                "fuzz_type": "%s (%s)"%(self.fuzz_type, self.job_name),
                "ticks": 0,
                "execs_per_second": 0.0,
                "num_inputs": 0,
                }
        r = self.update_client_info()
        self.client_info["id"]=int(r.content)
    
    def update_client_info(self):
        r_url = '%s/api/client_set_info/'%self.base_url
        r = requests.post(r_url, json=self.client_info)
        assert(r.reason=="OK")
        return r

    def add_input(self, path):
        r_url = '%s/api/client_add_input/'%self.base_url
        with open(path) as f:
            data = base64.b64encode(f.read())
        input_info = {
                "path": path,
                "raw_data": data
                }
        r = requests.post(r_url, json=input_info)
        print(r)

    def add_coverage_input(self, data_path, trace_path):
        r_url = '%s/api/client_add_coverage_and_input/'%self.base_url
        try:
            with open(data_path) as f:
                data = base64.b64encode(f.read())
        except:
            data = base64.b64encode("")
        with open(trace_path) as f:
            cov = f.read()
        input_info = {
                "path": data_path,
                "raw_data": data,
                "coverage": cov
                }
        r = requests.post(r_url, json=input_info)
        print(r)
