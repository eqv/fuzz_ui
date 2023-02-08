import sys
import glob
import os
import inotify.adapters
from inotify.constants import *
from api import IjonWebAPI

url = "http://127.0.0.1:8080/"

class AFLWatcher:
    @staticmethod
    def is_afl_dir(path):
        print(path)
        return os.path.exists(path+"/fuzzer_stats") and os.path.isdir(path+"/queue")

    def __init__(self, basepath):
        self.path = basepath
        self.known_inputs = set()
        self.api = IjonWebAPI("AFL", os.path.basename(basepath), url)
        self.api.register()

    def handle(self, path, filename, ev_types):
        if path.endswith("queue"): 
            self.handle_queue(path, filename, ev_types)
        if path.endswith("fuzzer_stats"):
            self.handle_stats(path, filename, ev_types)

    def handle_queue(self, path, filename, ev_types):
        self.send_queue_item(path+"/"+filename)

    def send_queue_item(self, path):
        if path not in self.known_inputs:
            self.known_inputs.add(path)
            self.api.add_input(path)

    def handle_stats(self, path, filename, ev_types):
        self.send_stats()
    
    def send_stats(self):
        data = {}
        with open(self.path+"/fuzzer_stats") as f:
            for l in f.readlines():
                field,val = l.split(":")
                data[field.strip()]=val.strip()
        self.api.client_info["ticks"] = int(data["last_update"]) - int(data["start_time"])
        self.api.client_info["execs_per_second"] = float(data["execs_per_sec"])
        self.api.client_info["num_inputs"] = int(data["paths_total"])
        self.api.update_client_info()

    def scan(self):
        self.send_stats()
        for p in glob.glob(self.path+"/queue/*"):
            self.send_queue_item(p)


class NYXWatcher:
    @staticmethod
    def is_nyx_dir(path):
        # Todo @ Cornelius
        return True


    def __init__(self, basepath):
        self.path = basepath
        self.known_inputs = set()
        self.api = IjonWebAPI("NYX", os.path.basename(basepath), url)
        self.api.register()

    def handle(self, path, filename, ev_types):
        if filename.endswith(".trace"):
            if 'IN_CLOSE_WRITE' in ev_types:
                print("adding %s and %s"%(path + filename.replace(".trace",".py"), path + filename))
                self.api.add_coverage_input(path + filename.replace(".trace",".py"), path + filename)

    def scan(self):
        for g in glob.glob(self.path+"/cov_*.trace"):
            print("adding %s and %s"%(g.replace(".trace",".py"),g))
            self.api.add_coverage_input(g.replace(".trace",".py"),g)



class Watcher:
    def __init__(self, path):
        self.inot = inotify.adapters.Inotify()
        self.handlers = {}
        if AFLWatcher.is_afl_dir(path):
            afl = AFLWatcher(path)
            mask_modify = IN_MOVED_FROM | IN_MOVED_TO | IN_CREATE  | IN_DELETE | IN_DELETE_SELF | IN_MOVE_SELF | IN_CLOSE_WRITE
            self.add_watcher(path, mask_modify, afl)
            self.add_watcher(path+"/queue", mask_modify, afl)
            self.add_watcher(path+"/fuzzer_stats", mask_modify, afl)
            afl.scan()
        if NYXWatcher.is_nyx_dir(path):
            nyx = NYXWatcher(path)
            mask_modify = IN_MOVED_FROM | IN_MOVED_TO | IN_CREATE  | IN_DELETE | IN_DELETE_SELF | IN_MOVE_SELF | IN_CLOSE_WRITE
            self.add_watcher(path, mask_modify, nyx)
            nyx.scan()
        else:
            print("warning, not a known workdir dir, exiting")
            exit(1)

    def add_watcher(self, path, mask, handler):
            self.inot.add_watch(path, mask = mask)
            self.handlers[path]=handler

    def run(self):
        for event in self.inot.event_gen(yield_nones=False):
            (_, type_names, path, filename) = event
            if type_names != ['IN_CLOSE_NOWRITE'] and type_names!= ['IN_ACCESS'] and type_names != ['IN_OPEN']:
                if path in self.handlers:
                    self.handlers[path].handle(path, filename, type_names)
                else:
                    print("PATH=[{}] FILENAME=[{}] EVENT_TYPES={}".format(path, filename, type_names))



if sys.argv[1] == "--add_inputs":
    for g in glob.glob(sys.argv[2]+"/*"):

        api = IjonWebAPI("importer", "some_files", url)
        api.register()
        api.add_input(g)

if sys.argv[1] == "--add_coverage":
    api = IjonWebAPI("importer", "some_files", url)
    api.register()
    for g in glob.glob(sys.argv[2]+"/cov_*.trace"):
        print("adding %s and %s"%(g.replace(".trace",".py"),g))
        api.add_coverage_input(g.replace(".trace",".py"),g)

elif sys.argv[1] == "--watch":
    w = Watcher(sys.argv[2])
    w.run()
else:
    print("usage: python watcher.py {--add_inputs,--add_coverage,--watch} /path/to/stuff")
