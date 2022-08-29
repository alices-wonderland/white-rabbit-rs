#!/usr/bin/env groovy

pipeline {
  agent {
    kubernetes {
      defaultContainer 'context'
      yaml '''
        apiVersion: v1
        kind: Pod
        spec:
          containers:
          - name: context
            image: node:18.8.0-alpine
            imagePullPolicy: Always
            command:
            - cat
            tty: true
            resources:
              limits:
                memory: "1400Mi"
                cpu: "600m"
              requests:
                memory: "700Mi"
                cpu: "300m"
          - name: kaniko
            image: gcr.io/kaniko-project/executor:latest
            imagePullPolicy: Always
            command:
            - cat
            tty: true
            resources:
              limits:
                memory: "1400Mi"
                cpu: "400m"
              requests:
                memory: "700Mi"
                cpu: "200m"
        '''
    }
  }
  stages {
    stage("Prepare Environment") {
      steps {
        sh 'apk update'
        sh 'apk upgrade'
        sh 'apk add --no-cache build-base webkit2gtk-dev curl wget openssl-dev gtk+3.0-dev libayatana-appindicator-dev librsvg-dev rust cargo'
      }
    }

    stage('Test') {
      steps {
        sh 'yarn'
        sh 'yarn test'
      }
    }

    stage('Build Docker Image') {
      steps {
        container('kaniko') {
          sh '/kaniko/executor --dockerfile packages/endpoint-graphql/Dockerfile'
        }
      }
    }
  }
}
