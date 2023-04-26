import * as pulumi from "@pulumi/pulumi";
import * as docker from "@pulumi/docker";
import * as k8s from "@pulumi/kubernetes";
import { DjinnServer } from "./apps/djinn-server";

new DjinnServer();
