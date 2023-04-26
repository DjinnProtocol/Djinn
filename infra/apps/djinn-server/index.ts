import * as pulumi from "@pulumi/pulumi";
import * as docker from "@pulumi/docker";
import * as k8s from "@pulumi/kubernetes";

const username = process.env.DOCKER_USERNAME;
const password = process.env.DOCKER_PASSWORD;

// Create registry
const registry = {
    username,
    password,
    server: "registry.nykaworks.com",
};

export class DjinnServer extends pulumi.ComponentResource {
    image: docker.Image;
    namespace: k8s.core.v1.Namespace;
    deployment: k8s.apps.v1.Deployment;
    dockerSecret: k8s.core.v1.Secret;
    service: k8s.core.v1.Service;

    constructor() {
        super("djinn-server", "djinn-server", {});

        this.image = new docker.Image("djinn-server-image", {
            build: {
                context: "../",
                dockerfile: "../djinn_server/Dockerfile",
                platform: "linux/amd64",
            },
            imageName: "registry.nykaworks.com/djinn_server:latest",
            registry,
        });

        this.namespace = new k8s.core.v1.Namespace("djinn-server-namespace", {
            metadata: {
                name: "djinn-server",
            },
        });

        this.dockerSecret = new k8s.core.v1.Secret("dockersecret", {
            metadata: {
                name: "dockersecret",
                namespace: this.namespace.metadata.name,
            },
            type: "kubernetes.io/dockerconfigjson",
            data: {
                ".dockerconfigjson": Buffer.from(JSON.stringify({
                    auths: {
                        [registry.server]: {
                            username: registry.username,
                            password: registry.password,
                            email: "",
                        },
                    },
                })).toString("base64"),
            },
        });

        const appLabels = { app: "djinn-server" };

        // this.configMap = new k8s.core.v1.ConfigMap(
        //     "djinn-server-configmap",

        //     {
        //         metadata: {
        //             namespace: this.namespace.metadata.name,
        //             labels: appLabels,
        //             name: "djinn-server-configmap",
        //         },
        //         data: {
        //             "RUST_LOG": "debug"
        //         }
        //     }
        // );


        this.deployment = new k8s.apps.v1.Deployment(
            "djinn-server-deployment",
            {
                metadata: {
                    namespace: this.namespace.metadata.name,
                },
                spec: {
                    selector: {
                        matchLabels: appLabels,
                    },
                    replicas: 1,
                    template: {
                        metadata: {
                            labels: appLabels,
                        },
                        spec: {
                            containers: [
                                {
                                    name: "djinn-server",
                                    image: this.image.imageName,
                                    ports: [
                                        {
                                            containerPort: 7777,
                                            name: "djinn",
                                        },
                                    ],
                                    env: [
                                        {
                                            name: "RUST_LOG",
                                            value: "debug"
                                        }
                                    ]
                                },
                            ],
                            imagePullSecrets: [
                                {
                                    name: this.dockerSecret.metadata.name
                                }
                            ]
                        },
                    },
                },
            }
        );

        this.service = new k8s.core.v1.Service("djinn-server-service", {
            metadata: {
                namespace: this.namespace.metadata.name,
            },
            spec: {
                type: "NodePort",
                selector: this.deployment.spec.template.metadata.labels,
                ports: [
                    {
                        port: 7777,
                        targetPort: 7777,
                        protocol: "TCP",
                        name: "djinn",
                        nodePort: 30777
                    }
                ]
            }
        });
    }
}
