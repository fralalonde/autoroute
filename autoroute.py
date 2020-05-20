#!/usr/bin/python3

# USB MIDI persitent patchbay using ALSA sequencer
# developed for Raspberry Pi but could run elesewhere
# requires python 3.5 and alsa-utils
# can run as a service, triggered on boot and when new USB MIDI device is plugged in

import sys, os, io, subprocess, re
from shutil import copy

# args parsing

ARGS = iter([arg for arg in sys.argv if not arg.startswith('-')])
OPTS = dict([opt.split('=') for opt in sys.argv if opt.startswith('-') and '=' in opt])

# FLAGS = set([flag for flag in sys.argv if flag.startswith('-') and '=' not in flag])

APP_NAME = next(ARGS)


def expected_arg(name):
    try:
        return next(ARGS)
    except:
        print('Error: Missing argument "{}".'.format(name))
        print_usage()
        sys.exit(-1)


def print_usage():
    print("autoroute (list | connect [--config=configfile] | install)")
    print("default config file is autoroute.conf")


device_regex = re.compile("^client (\d+):")
port_name_regex = re.compile("^\s+(\d+)\s+'(.+)'")
ports = {}
connect_fwd = {}

# load config
ignore_regex = re.compile("^ignore\s+(.+)$")
connect_regex = re.compile("^connect\s+(.+)\s*->\s*(.+)$")
ignore = {}


def load_ports():
    aconnect = subprocess.Popen(['aconnect', '-l'], stdout=subprocess.PIPE)
    device_no = -1
    device_name = -1
    for line in aconnect.stdout:
        line = line.decode('utf-8')
        device_match = device_regex.match(line)
        if device_match:
            device_no = device_match.group(1)
            continue
        port_match = port_name_regex.match(line)
        if port_match:
            port_name = port_match.group(2).strip()
            if port_name not in ignore:
                ports[port_name] = device_no + ":" + port_match.group(1)


def list_ports():
    load_ports()
    for dev in ports.keys():
        print(ports[dev] + ' ' + dev)


def load_config():
    # Require ports 
    load_ports()
    # Forward connection map
    conf = OPTS.get('--config', 'autoroute.conf')
    with open(conf, 'r') as conf:
        for line in conf:
            match = ignore_regex.match(line)
            if match:
                ignored = match.group(1).strip()
                ignore[ignored] = ''
                continue

            match = connect_regex.match(line)
            if match:
                source = match.group(1).strip()
                dest = match.group(2).strip()

                if source not in connect_fwd:
                    connect_fwd[source] = [dest]
                else:
                    connect_fwd[source].append(dest)


def connect_ports():
    load_config()
    subprocess.Popen(['aconnect', '-x'])
    for source in connect_fwd.keys():
        if source not in ports:
            print('Device "' + source + '" not present, ignored')
            continue

        for dest in connect_fwd[source]:
            if dest not in ports:
               print('Device "' + dest + '" not present, ignored')
               continue

            print('Connecting ' + source + ' [' + ports[source] + '] to ' + dest + ' [' + ports[dest] + ']')
            subprocess.Popen(['aconnect', ports[source], ports[dest]])


def install_service():
    with open('/etc/udev/rules.d/33-midiusb.rules', 'w') as file:
        file.write('ACTION=="add|remove", SUBSYSTEM=="usb", DRIVER=="usb", RUN+="/usr/local/bin/autoroute connect --config=/etc/autoroute.conf"\n')

    print('/etc/udev/rules.d/33-midiusb.rules')

    with open('/lib/systemd/system/midi.service', 'w') as file:
        file.write('[Unit]\n')
        file.write('Description=Initial USB MIDI connect\n')
        file.write('[Service]\n')
        file.write('ExecStart=/usr/local/bin/autoroute connect --config=/etc/autoroute.conf\n')
        file.write('[Install]\n')
        file.write('WantedBy=multi-user.target\n')
    print('/lib/systemd/system/midi.service')

    copy(APP_NAME, '/usr/local/bin/')
    print('/usr/local/bin/' + APP_NAME)

    subprocess.Popen(['systemctl', 'enable', 'midi.service'])
    subprocess.Popen(['udevadm', 'control', '--reload'])
    subprocess.Popen(['service', 'udev', 'restart'])


action = expected_arg('action')

if action == 'connect':
    connect_ports()

elif action == 'list':
    list_ports()

elif action == 'install':
    install_service()

else:
    print('Error: Unknown action "{}".'.format(action))
    print_usage()
