# Throwaway ansible control-plane image for deploying dropvoice pairing-server
# from Windows. Local-only tool — never pushed. Built on first use, cached after.
#
# Why a container: Windows cannot run ansible natively (ansible-core requires
# POSIX fork/sh). This image provides:
#   - ansible-core + community.docker collection (docker_compose_v2 module)
#   - OpenSSH client (control node -> target host)
#
# The repo (playbooks/templates/vaults) is bind-mounted at /workspace, so
# playbook edits need no image rebuild.

FROM python:3.12-slim

# Version pins: ansible-core pinned here (community.docker requires >=2.17;
# 2.18.* per requirements.yml), the collection comes from requirements.yml so
# that file stays the auditable single source for collection versions.
COPY requirements.yml /tmp/requirements.yml
RUN pip install --no-cache-dir 'ansible-core==2.18.*' && \
    ansible-galaxy collection install -r /tmp/requirements.yml

RUN apt-get update && \
    apt-get install -y --no-install-recommends openssh-client ca-certificates git && \
    rm -rf /var/lib/apt/lists/*

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

WORKDIR /workspace
ENTRYPOINT ["/entrypoint.sh"]
