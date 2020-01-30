# RoleBinding for modules.

We have discovered that in some clusters, all cluster resources are gated by RBAC. Azure IoT Edge 
on Kubernetes assigns ServiceAccounts to Pods for [module identification](rbac.md#module-authentication). 
Because these module ServiceAccounts have no roles bound to them, they are given minimal permissions. 
Our customers need to assign roles to do common tasks normally allowed in more permissive clusters, 
like mounting persistent volumes.

## Customer Responsibility

- understand cluster restrictions
- provide roles
- understand what roles apply to a module's needs
- Provide roles with sufficient permission to RoleBinding creator

## Azure IoT Edge responsibility

- Provide some mechanism to assign roles to a module.  This should be flexible, easy to understand, and (if required) set through edge deployment.
- If edge team provides an operator to assign roles, it should be optional, to allow for customer to provide their own role assignment operator or none at all.

[Recommended, documented K8s strategies](https://kubernetes.io/docs/reference/access-authn-authz/rbac/#service-account-permissions) 
for resolving this from most to less secure:
1. Grant a role to an application-specific service account (best practice)
2. Grant a role to the “default” service account in a namespace
3. Grant a role to all service accounts in a namespace

#2 is not feasible with current design. We are not using the "default" ServiceAccount.

## Option 1. Grant a role to an application-specific service account
- This is something that requires design and code changes.

There are two parts to this solution:
1. Create a specification for assigning roles to modules.
2. Create an operator to assign roles to modules.

### Suggestions to specify role binding per module:

1. Add a labels section in "k8s-experimental" extensions.

This allows the user to apply labels to Pods, ServiceAccounts, or all objects created for module.

Labels are a flat dictionary, label keys and values are very strict in format, and duplicate keys are not allowed.
There would need to be some lookup - a label or set of labels would be chosen to apply to a specific RoleBinding. This would be defined by the operator.

- Generally useful to apply labels to pods and k8s objects - We may want this regardless, not just for binding roles.
- Can either be done by an operator or by EdgeAgent.
- Operator may define how labels map to roles independent of EdgeAgent.

2. Add annotations section in "k8s-experimental" extensions.

This allows the user to apply annotations to Pods, ServiceAccounts, or all objects created for module.

Annotations are more flexible in value content.  For example, we could give a comman-separated list of roles to bind a ServiceAccount.  An operator would still need to define a format for interpreting annotations into role bindings.

- Generally useful to apply annotations to objects.
- Role binding can either be done by an operator or by EdgeAgent.
- Operator may define how annotations map to roles independent of EdgeAgent.
- Using annotations implies storing a list of roles to bind a module's service account.

3. Use labels in module's createOptions. The labels in the createOptions are translated to annotations on the Pod. They could also be translated to annotations on the ServiceAccount or other created objects, if that is deemed necessary.

- same as above.

4. Use some combination of labels and annotations. Labels could be the selection criteria (label indicates a module needs a role binding) and annotation is the data store (annotation contains the roles to be bound).

Add a method for both specifying labels and annotations (if docker labels are inadequate)

- Can either be done by an operator or by EdgeAgent.
- Operator may define process of binding roles independent of EdgeAgent

5. Add roles section in "k8s-experimental" extensions.

- Just gives us an array of roles to bind to.
- This limits behavior to EdgeAgent - no independent operator.
- This section would not have the same behavior of the other sections in "k8s-experimental", this implies a weak design.

### Suggestions for automatically creating role bindings:

1. Agent creates the RoleBinding.

Either as part of the Edge Deployment Operator, or through some other Operator task, Agent creates the role binding from the information provied.
- This requires the Edge team to specify the rules of how a role is bound.
- This requires EdgeAgent to have more permission that it currently has, including additional permissions to create, list, and remove RoleBindings.

2. Out-of-band binding.

The deployment mechanism creates the RoleBinding for module's ServiceAccount as part of the deployent.
- No change to Edge Runtime.
- This implies the Edge Deployment configuration is known before the Edge Deployment is given.
- This is generally seen as undesireable.

3. A separate role binding operator created by the Edge team.

This application watches k8s objects, possibly with some label selection criteria, and creates and deletes RoleBindings.
- This app has a single responsibility.
- This requires the Edge team to specify the rules of how a role is bound.
- Can be installed as a cluster-wide operator.
- Can be optionally installed.

4. A separate role binding operator created by the customer.

Instead of the edge team developing an operator, the customer may also do the same.
- Does not necessarily require changes to Edge Runtime, but using labels or annotation may be desired.
- Customer has control over this responsibility, does not require the edge team to come up with binding rules.

## Option 3: Grant a role to all service accounts in a namespace
- No additional changes required for Edge on K8s.

https://kubernetes.io/docs/reference/access-authn-authz/rbac/
> If you want all applications in a namespace to have a role, no matter what service account they use, you can grant a role to the service account group for that namespace.

> For example, grant read-only permission within “my-namespace” to all service accounts in that namespace:

```sh
kubectl create rolebinding serviceaccounts-view \
  --clusterrole=view \
  --group=system:serviceaccounts:my-namespace \
  --namespace=my-namespace
```